use anyhow::Result;
use reqwest::multipart::{Form, Part};
use serde_json::{json, Value};
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::{fs::File, io::Read};

// ref https://pumpportal.fun/creation/
#[allow(clippy::too_many_arguments)]
pub async fn send_local_create_tx(
    wallet_private_base58: &str,
    avatar_file_path: &str,
    token_name: &str,
    token_symbol: &str,
    token_description: &str,
    staked_endpoint: &str,
    init_buy_amount: f32,
    slippage: u64,
    priority_fee: f32,
) -> Result<(String, String)> {
    // Initialize signer keypair from base58 string
    let signer_keypair = Keypair::from_base58_string(wallet_private_base58);

    // Generate a random keypair for token
    let mint_keypair = Keypair::new();
    // println!("Mint keypair: {}", mint_keypair.to_base58_string());

    // Read the image file

    let mut file = File::open(avatar_file_path)?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)?;

    // Create form data for metadata
    let form = Form::new()
        .text("name", token_name.to_string())
        .text("symbol", token_symbol.to_string())
        .text("description", token_description.to_string())
        .text("twitter", "https://x.com/a1lon9/status/1812970586420994083")
        .text(
            "telegram",
            "https://x.com/a1lon9/status/1812970586420994083",
        )
        .text("website", "https://pumpportal.fun")
        .text("showName", "true")
        .part(
            "file",
            Part::bytes(file_contents)
                .file_name("cat2.png")
                .mime_str("image/png")?,
        );

    // Create IPFS metadata storage
    let client = reqwest::Client::new();
    let metadata_response = client
        .post("https://pump.fun/api/ipfs")
        .multipart(form)
        .send()
        .await?;

    let metadata_json: Value = metadata_response.json().await?;
    let metadata_uri = metadata_json["metadataUri"].as_str().unwrap();

    // Generate the create transaction
    let create_tx_response = client
        .post("https://pumpportal.fun/api/trade-local")
        .json(&json!({
            "publicKey": signer_keypair.pubkey().to_string(),
            "action": "create",
            "tokenMetadata": {
                "name": token_name,
                "symbol": token_symbol,
                "uri": metadata_uri
            },
            "mint": mint_keypair.pubkey().to_string(),
            "denominatedInSol": "true",
            "amount": init_buy_amount, // Dev buy of 0.001 SOL
            "slippage": slippage,
            "priorityFee": priority_fee,
            "pool": "pump"
        }))
        .send()
        .await?;

    let tx_bytes = create_tx_response.bytes().await?;
    let mut versioned_tx = bincode::deserialize::<VersionedTransaction>(tx_bytes.as_ref())
        .map_err(anyhow::Error::msg)?;

    let rpc_client =
        RpcClient::new_with_commitment(staked_endpoint.to_string(), CommitmentConfig::confirmed());
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    // dbg!(versioned_tx.message.instructions());

    versioned_tx.message.set_recent_blockhash(recent_blockhash);
    let versioned_tx = VersionedTransaction::try_new(
        versioned_tx.message.clone(),
        &[&mint_keypair, &signer_keypair],
    );
    assert!(versioned_tx.is_ok());
    let versioned_tx = versioned_tx.unwrap();

    let tx_signature = rpc_client.send_and_confirm_transaction(&versioned_tx)?;
    println!("Transaction: https://solscan.io/tx/{}", tx_signature);

    Ok((mint_keypair.to_base58_string(), tx_signature.to_string()))
}

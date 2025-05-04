use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair};
use anyhow::Result;
use bincode::serialize;
use pumpfun::{utils::CreateTokenMetadata, PriorityFee, PumpFun};
use solana_sdk::{
    bs58::encode,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Signer,
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};
use std::str::FromStr;

pub async fn build_buy_instructions(
    wallet_private_base58: &str,
    mint_pubkey: &str,
    amount_sol: f32,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
    cluster_url: Option<&str>,
) -> Result<Vec<Instruction>> {
    if wallet_private_base58.is_empty() {
        return Err(anyhow::Error::msg("Wallet private key is empty"));
    }
    if mint_pubkey.is_empty() {
        return Err(anyhow::Error::msg("Mint pubkey is empty"));
    }
    if amount_sol <= 0.0 {
        return Err(anyhow::Error::msg("Amount of SOL must be greater than 0.0"));
    }

    // payer pay for PDA's Account
    let payer = Keypair::from_base58_string(wallet_private_base58);
    let mint_pubkey = Pubkey::from_str(mint_pubkey)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing mint pubkey: {:?}", e)))?;

    let cluster = match cluster_url {
        Some(url) => anchor_client::Cluster::Custom(
            url.to_string(),
            "wss://api.mainnet-beta.solana.com".to_string(),
        ),
        None => anchor_client::Cluster::Mainnet,
    };
    let client: PumpFun = PumpFun::new(cluster, payer, None, None);
    let amount_lamports: u64 = (LAMPORTS_PER_SOL as f32 * amount_sol) as u64;
    let instructions = client
        .buy(
            &mint_pubkey,
            amount_lamports,
            slippage_basis_points,
            priority_fee,
        )
        .await
        .map_err(|e| anyhow::Error::msg(format!("Error buying tokens: {:?}", e)))?;
    Ok(instructions)
}

#[allow(clippy::too_many_arguments)]
pub async fn build_create_and_buy_instructions(
    wallet_private_base58: &str,
    token_name: &str,
    token_symbol: &str,
    token_description: &str,
    token_file: &str,
    twitter: &str,
    telegram: &str,
    website: &str,
    amount_sol: f32,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
    cluster_url: Option<&str>,
) -> Result<(Vec<Instruction>, Keypair)> {
    let payer = Keypair::from_base58_string(wallet_private_base58);
    let mint_keypair = Keypair::new();
    dbg!(mint_keypair.to_base58_string());
    dbg!(mint_keypair.pubkey());
    let create_token_metadata = CreateTokenMetadata {
        name: token_name.to_string(),
        symbol: token_symbol.to_string(),
        description: token_description.to_string(),
        file: token_file.to_string(),
        twitter: Some(twitter.to_string()),
        telegram: Some(telegram.to_string()),
        website: Some(website.to_string()),
    };
    let cluster = match cluster_url {
        Some(url) => anchor_client::Cluster::Custom(
            url.to_string(),
            "wss://api.mainnet-beta.solana.com".to_string(),
        ),
        None => anchor_client::Cluster::Mainnet,
    };
    let client = PumpFun::new(cluster, payer, None, None);
    let amount_lamports: u64 = (LAMPORTS_PER_SOL as f32 * amount_sol) as u64;
    let instructions = client
        .create_and_buy(
            &mint_keypair,
            create_token_metadata,
            amount_lamports,
            slippage_basis_points,
            priority_fee,
        )
        .await
        .map_err(|e| anyhow::Error::msg(format!("Error create token: {:?}", e)))?;
    Ok((instructions, mint_keypair))
}

#[derive(Debug)]
pub struct BuyTx {
    pub key: String,
    pub lamports: u64, // 1 lamport = 0.000000001 SOL
    pub slippage: u64,
    // pub jito_tips: u64, // 1 jito_tip = 0.000000001 SOL
}

#[derive(Debug)]
pub struct SellTx {
    pub key: String,
    pub amount: Option<u64>, // 1 lamport = 0.000000001 SOL
    pub slippage: u64,
}

pub const JITO_TIP_PUBKEY_MAINNET: &str = "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt";

pub async fn build_bundle_buy_batch(
    mint_key: &str,
    tx_params: &Vec<BuyTx>,
    cluster_url: Option<&str>,
) -> Result<Vec<String>> {
    let mint_pubkey = Pubkey::from_str(mint_key)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing mint pubkey: {:?}", e)))?;
    let jito_tip_pubkey = Pubkey::from_str(JITO_TIP_PUBKEY_MAINNET)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing jito tip pubkey 6: {:?}", e)))?;

    let mut bundle_txs: Vec<String> = vec![];
    for tx_param in tx_params {
        let tx_str =
            build_bundle_buy_single(&mint_pubkey, tx_param, &jito_tip_pubkey, cluster_url).await?;
        bundle_txs.push(tx_str);
    }

    Ok(bundle_txs)
}

pub async fn build_bundle_buy_single(
    mint_pubkey: &Pubkey,
    tx_param: &BuyTx,
    jito_tip_pubkey: &Pubkey,
    cluster_url: Option<&str>,
) -> Result<String> {
    let wallet_private_base58 = &tx_param.key;
    if wallet_private_base58.is_empty() {
        return Err(anyhow::Error::msg("Wallet private key is empty"));
    }
    let payer = Keypair::from_base58_string(wallet_private_base58);
    let cluster = match cluster_url {
        Some(url) => anchor_client::Cluster::Custom(
            url.to_string(),
            "wss://api.mainnet-beta.solana.com".to_string(),
        ),
        None => anchor_client::Cluster::Mainnet,
    };
    let priority_fee = PriorityFee {
        limit: Some(63909),
        price: Some(140999), // Set 0.14 lamports per compute unit
    };
    let client: PumpFun = PumpFun::new(cluster, payer.insecure_clone(), None, None);

    let mut instructions = vec![];
    instructions.extend(
        client
            .buy(
                mint_pubkey,
                tx_param.lamports,
                Some(tx_param.slippage),
                Some(priority_fee),
            )
            .await
            .map_err(|e| anyhow::Error::msg(format!("Error buying tokens: {:?}", e)))?,
    );
    // jito tips 0.00001 sol
    instructions.push(transfer(&payer.pubkey(), jito_tip_pubkey, 10_000));

    let tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[&payer],
        client.rpc.get_latest_blockhash()?,
    ));
    let serialized_tx: Vec<u8> = serialize(&tx)?;
    let transaction_base58: String = encode(&serialized_tx).into_string();

    Ok(transaction_base58)
}

pub async fn build_bundle_sell_single(
    mint_pubkey: &Pubkey,
    tx_param: &SellTx,
    jito_tip_pubkey: &Pubkey,
    cluster_url: Option<&str>,
    close_token_ata: bool,
) -> Result<String> {
    let wallet_private_base58 = &tx_param.key;
    if wallet_private_base58.is_empty() {
        return Err(anyhow::Error::msg("Wallet private key is empty"));
    }
    let payer = Keypair::from_base58_string(wallet_private_base58);
    let cluster = match cluster_url {
        Some(url) => anchor_client::Cluster::Custom(
            url.to_string(),
            "wss://api.mainnet-beta.solana.com".to_string(),
        ),
        None => anchor_client::Cluster::Mainnet,
    };
    let priority_fee = PriorityFee {
        limit: Some(63909),
        price: Some(140999), // Set 0.14 lamports per compute unit
    };
    let client: PumpFun = PumpFun::new(cluster, payer.insecure_clone(), None, None);

    let mut instructions = vec![];
    instructions.extend(
        client
            .sell(
                mint_pubkey,
                tx_param.amount,
                Some(tx_param.slippage),
                Some(priority_fee),
                close_token_ata,
            )
            .await
            .map_err(|e| anyhow::Error::msg(format!("Error selling tokens: {:?}", e)))?,
    );
    // jito tips 0.00001 sol
    instructions.push(transfer(&payer.pubkey(), jito_tip_pubkey, 10_000));

    let tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[&payer],
        client.rpc.get_latest_blockhash()?,
    ));
    let serialized_tx: Vec<u8> = serialize(&tx)?;
    let transaction_base58: String = encode(&serialized_tx).into_string();

    Ok(transaction_base58)
}

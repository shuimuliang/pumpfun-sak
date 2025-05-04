use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};

pub fn transfer_sol(
    rpc_client: &RpcClient,
    from_keypair: &Keypair,
    to_pubkey: &solana_sdk::pubkey::Pubkey,
    lamports: u64,
) -> anyhow::Result<()> {
    let block_hash = rpc_client.get_latest_blockhash().unwrap();

    let transfer_instruction =
        system_instruction::transfer(&from_keypair.pubkey(), to_pubkey, lamports);
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&from_keypair.pubkey()));
    transaction.sign(&[from_keypair], block_hash);

    let sig = rpc_client.send_and_confirm_transaction(&transaction)?;
    dbg!(&sig);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let staked_endpoint = "https://mainnet.helius-rpc.com/?api-key=";
    let rpc_client = RpcClient::new(staked_endpoint.to_string());

    let wallet_private_base58 = "";
    let payer = Keypair::from_base58_string(wallet_private_base58);

    dbg!(payer.pubkey());

    // Specify the amount to transfer
    let lamports_to_transfer: u64 = 1000000; // 1 sol = 10^8 lamports, 0.01
    let to_pubkey: Pubkey = ""
        .parse()
        .unwrap();

    transfer_sol(&rpc_client, &payer, &to_pubkey, lamports_to_transfer)?;

    let alice_balance = rpc_client.get_balance(&payer.pubkey())?;
    let dave_balance = rpc_client.get_balance(&to_pubkey)?;
    dbg!(alice_balance);
    dbg!(dave_balance);

    Ok(())
}

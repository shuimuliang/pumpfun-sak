use bincode::serialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    bs58::encode,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};
use std::str::FromStr;

const JITO_TIP_PUBKEY_MAINNET: &str = "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt";
const JITO_TIP_PUBKEY_TESTNET: &str = "ARTtviJkLLt6cHGQDydfo1Wyk6M4VGZdKZ2ZhdnJL336";

pub async fn build_simple_transfer_bundle(
    from_keypair: &str,
    to: &[&str],
    amount: u64,
    is_testnet: bool,
) -> anyhow::Result<Vec<String>> {
    let mut txs: Vec<String> = vec![];

    let rpc_url = if is_testnet {
        "https://api.testnet.solana.com"
    } else {
        "https://mainnet.helius-rpc.com/?api-key=5d166540-f22e-4f66-bb70-8349844d4a0e"
        // "https://api.mainnet-beta.solana.com"
    };
    let rpc_client = RpcClient::new(rpc_url.to_string());

    let from_keypair = Keypair::from_base58_string(from_keypair);

    for recipient in to {
        let mut ixs = vec![];

        // Set 1,000 compute units
        let compute_units_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(1_000);
        // Set 0.01 lamports per compute unit
        let compute_budget_ix: Instruction =
            ComputeBudgetInstruction::set_compute_unit_price(10_000);
        ixs.push(compute_units_ix);
        ixs.push(compute_budget_ix);

        let jito_tip_pubkey = if is_testnet {
            JITO_TIP_PUBKEY_TESTNET
        } else {
            JITO_TIP_PUBKEY_MAINNET
        };
        ixs.push(transfer(
            &from_keypair.pubkey(),
            &Pubkey::from_str(recipient)?,
            amount,
        ));
        // jito tips
        // 0.00005 sol
        let tip = 50_000;
        ixs.push(transfer(
            &from_keypair.pubkey(),
            &Pubkey::from_str(jito_tip_pubkey)?,
            tip,
        ));

        // transfer instruction
        let tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
            &ixs,
            Some(&from_keypair.pubkey()),
            &[&from_keypair],
            rpc_client.get_latest_blockhash().await?,
        ));

        let serialized_tx: Vec<u8> = serialize(&tx)?;
        let transaction_base58: String = encode(&serialized_tx).into_string();

        txs.push(transaction_base58);
        println!(
            "Sending {} lamports from {} to {}",
            amount,
            from_keypair.pubkey(),
            recipient
        );
    }

    Ok(txs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_build_sol_bundle() {
        let from_keypair = "";
        let to = [
            "",
            "",
        ];
        let amount = 1000;
        let is_testnet = true;
        let res = build_simple_transfer_bundle(from_keypair, &to, amount, is_testnet).await;
        assert!(res.is_ok());

        let txs = res.unwrap();
        assert_eq!(txs.len(), 2);
    }
}

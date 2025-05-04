use anyhow::Result;
use helius::types::types::{Cluster, CreateSmartTransactionSeedConfig, Timeout};
use helius::{Helius, HeliusFactory};
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction::transfer,
};
use std::str::FromStr;

// Replace lifetimes with Arc for CreateSmartTransactionConfig
// https://github.com/helius-labs/helius-rust-sdk/pull/89/files

#[tokio::main]
async fn main() -> Result<()> {
    let from_keypair: Keypair = Keypair::from_base58_string("");
    let from_pubkey: Pubkey = from_keypair.pubkey();
    let to_pubkey: Pubkey =
        Pubkey::from_str("").unwrap();
    let from_seeds = from_keypair.secret().to_bytes();

    let handle1 = tokio::spawn(async move {
        let factory: HeliusFactory = HeliusFactory::new("5d166540-f22e-4f66-bb70-8349844d4a0e");
        let helius: Helius = factory.create(Cluster::MainnetBeta).unwrap();

        // Create a simple instruction (transfer 0.01 SOL from from_pubkey to to_pubkey)
        let transfer_amount: u64 = 10_000; // 0.01 SOL in lamports
        let instructions: Vec<Instruction> =
            vec![transfer(&from_pubkey, &to_pubkey, transfer_amount)];

        let create_config: CreateSmartTransactionSeedConfig = CreateSmartTransactionSeedConfig {
            instructions,
            signer_seeds: vec![from_seeds],
            lookup_tables: None,
            fee_payer_seed: None,
            priority_fee_cap: None,
        };

        let send_options = RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        };

        // Send the optimized transaction with a 10k lamport tip using the New York region's API URL
        match helius
            .send_smart_transaction_with_seeds(
                create_config,
                Some(send_options),
                Some(Timeout::default()),
            )
            .await
        {
            Ok(sig) => {
                println!("https://explorer.solana.com/tx/{}", sig);
            }
            Err(e) => {
                eprintln!("Failed to send transaction: {:?}", e);
            }
        }
    });

    let result = handle1.await;

    match result {
        Ok(_) => println!("ok"),
        Err(err) => println!("error: {}", err),
    }

    Ok(())
}

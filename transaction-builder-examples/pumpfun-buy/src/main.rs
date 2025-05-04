use helius::types::types::{CreateSmartTransactionSeedConfig, Timeout};
use helius::{Helius, HeliusFactory};
use pumpfun_transaction_builder::build_buy_instructions;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() {
    let wallet_private_base58 = "";
    let mint_pubkey = "GP7gx56VH3g5mAei5n2VaDfVV6BRkJ6GiSHLhE6ypump";
    let amount_sol = 0.0001;
    let slippage: u64 = 0;
    let slippage_basis_points: Option<u64> = match slippage {
        0 => None,
        _ => Some(slippage * 100), // (1 basic point = 0.01%). Defaults to 500
    };

    let cluster_url =
        "https://mainnet.helius-rpc.com/?api-key=5d166540-f22e-4f66-bb70-8349844d4a0e";
    let instructions = build_buy_instructions(
        wallet_private_base58,
        mint_pubkey,
        amount_sol,
        slippage_basis_points,
        None,
        Some(cluster_url),
    )
    .await;
    assert!(instructions.is_ok());
    let instructions = instructions.unwrap();

    let buyer_keypair: Keypair = Keypair::from_base58_string(wallet_private_base58);
    let buyer_seeds = buyer_keypair.secret().to_bytes();

    let handle = tokio::spawn(async move {
        let factory: HeliusFactory = HeliusFactory::new("5d166540-f22e-4f66-bb70-8349844d4a0e");
        let helius: Helius = factory
            .create(helius::types::types::Cluster::MainnetBeta)
            .unwrap();

        let create_config: CreateSmartTransactionSeedConfig = CreateSmartTransactionSeedConfig {
            instructions,
            signer_seeds: vec![buyer_seeds],
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
        };
    });
    let result = handle.await;

    match result {
        Ok(_) => println!("ok"),
        Err(err) => println!("error: {}", err),
    }
}

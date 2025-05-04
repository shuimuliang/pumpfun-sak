use helius::types::types::Cluster;
use helius::Helius;
use pumpfun_transaction_builder::{build_bundle_sell_single, SellTx, JITO_TIP_PUBKEY_MAINNET};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key: &str = "<api-key>";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster)?;

    let mint_key = "FMcTudc98geTV75he1xuPqMNKdqBU4PdsT55pmuCpump";
    let mint_key = Pubkey::from_str(mint_key).unwrap();

    let jito_tips = Pubkey::from_str(JITO_TIP_PUBKEY_MAINNET).unwrap();

    // dave
    let sell_tx = SellTx {
        key: "".to_string(),
        amount: None,
        slippage: 500, // (1 bp = 0.01%). Defaults to 500
                       // jito_tips: 50_000, // 0.00005 sol
    };
    let close_token_ata = true;
    let cluster_url = "https://mainnet.helius-rpc.com/?api-key=<api-key>";
    let res = build_bundle_sell_single(
        &mint_key,
        &sell_tx,
        &jito_tips,
        Some(cluster_url),
        close_token_ata,
    )
    .await;
    assert!(res.is_ok());

    let txs = res.unwrap();
    let jito_api_url = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

    match helius.send_jito_bundle(vec![txs], jito_api_url).await {
        Ok(bundle_id) => {
            println!("Transaction sent successfully: {}", bundle_id);
            let res = helius
                .get_bundle_statuses(vec![bundle_id], jito_api_url)
                .await?;
            println!("{:?}", res);
        }
        Err(e) => {
            eprintln!("Failed to send transaction: {:?}", e);
        }
    }
    Ok(())
}

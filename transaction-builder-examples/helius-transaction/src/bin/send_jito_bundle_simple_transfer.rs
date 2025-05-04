use helius::types::types::Cluster;
use helius::Helius;
use sol_bundle_builder::build_simple_transfer_bundle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key: &str = "";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster)?;

    let from_keypair = "";
    let to = [
        "",
        "",
    ];
    let amount = 1000;
    let is_testnet = false;
    let res = build_simple_transfer_bundle(from_keypair, &to, amount, is_testnet).await;
    let txs = res.unwrap();

    let jito_api_url = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

    match helius.send_jito_bundle(txs, jito_api_url).await {
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

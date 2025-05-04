use helius::{error::Result, types::Cluster, Helius};
use solana_program::pubkey;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "5d166540-f22e-4f66-bb70-8349844d4a0e";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new_with_ws(api_key, cluster).await.unwrap();
    dbg!(&helius.config.endpoints);
    dbg!(&helius.config.cluster);
    let key: pubkey::Pubkey = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");

    if let Some(ws) = helius.ws() {
        let (mut stream, _unsub) = ws.account_subscribe(&key, None).await?;
        while let Some(event) = stream.next().await {
            println!("{:#?}", event);
        }
    }

    Ok(())
}

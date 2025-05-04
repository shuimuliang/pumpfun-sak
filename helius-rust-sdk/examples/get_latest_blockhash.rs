use helius::error::Result;
use helius::types::*;
use helius::Helius;

use solana_client::client_error::ClientError;
use solana_sdk::hash::Hash;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let result: std::result::Result<Hash, ClientError> = helius.connection().get_latest_blockhash();
    println!("{:?}", result);

    Ok(())
}

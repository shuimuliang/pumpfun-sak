use helius::error::Result;
use helius::types::{Cluster, Webhook};
use helius::{Helius, HeliusFactory};

#[tokio::main]
async fn main() {
    let factory: HeliusFactory = HeliusFactory::new("");
    let helius1: Helius = factory.create(Cluster::MainnetBeta).unwrap();
    let result: Result<Vec<Webhook>> = helius1.get_all_webhooks().await;
    println!("{:?}", result.unwrap());

    let helius2: Helius = factory.create(Cluster::MainnetBeta).unwrap();
    let result: Result<Vec<Webhook>> = helius2.get_all_webhooks().await;
    println!("{:?}", result.unwrap());
}

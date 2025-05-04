use clap::Parser;
use helius::{
    error::Result,
    types::{
        Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions,
    },
    Helius,
};
use helius_ws_hooks::pumpfun_instruction_parser::parse_notification;
use rdkafka::{
    config::ClientConfig,
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};
use solana_program::pubkey;
use std::str::FromStr;
use std::time::Duration;
use tokio_stream::StreamExt;

#[derive(Parser)]
struct Args {
    /// The program ID to subscribe to
    #[clap(default_value = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")]
    program_id: String,
    #[clap(default_value = "5d166540-f22e-4f66-bb70-8349844d4a0e")]
    api_key: String,
    #[clap(default_value = "pumpfun-create")]
    topic_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let program_id = pubkey::Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let api_key = args.api_key.clone();
    let topic_name = args.topic_name;

    // init helius
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new_with_ws_with_timeouts(&api_key, cluster, Some(15), Some(45))
        .await
        .unwrap();
    let config: RpcTransactionsConfig = RpcTransactionsConfig {
        filter: TransactionSubscribeFilter::standard(&program_id),
        options: TransactionSubscribeOptions::default(),
    };

    if let Some(ws) = helius.ws() {
        // init kafka
        let brokers = "localhost";
        let producer: &FutureProducer = &ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        let (mut stream, _unsub) = ws.transaction_subscribe(config).await.unwrap();
        while let Some(notification) = stream.next().await {
            if let Some(wrap_payload) = parse_notification(&notification, &program_id.to_string()) {
                let topic_name = topic_name.clone();
                let futures = async move {
                    // The send operation on the topic returns a future, which will be
                    // completed once the result or failure from Kafka is received.
                    producer
                        .send(
                            FutureRecord::to(&topic_name)
                                .payload(&format!(
                                    "Payload: {}",
                                    serde_json::to_string(&wrap_payload).unwrap()
                                ))
                                .key(&format!("Key {}", 0))
                                .headers(OwnedHeaders::new().insert(Header {
                                    key: "header_key",
                                    value: Some("header_value"),
                                })),
                            Duration::from_secs(0),
                        )
                        .await
                };
                let _t = futures.await;
            }
        }
    }

    Ok(())
}

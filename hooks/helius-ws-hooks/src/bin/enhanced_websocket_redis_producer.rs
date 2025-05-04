use chrono::prelude::Utc;
use clap::Parser;
use helius::{
    error::{HeliusError, Result},
    types::{
        Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions,
    },
    Helius,
};
use lazy_static::lazy_static;
use redis::AsyncCommands;
use solana_program::pubkey;
use std::str::FromStr;
use tokio::time::{sleep, timeout, Duration};
use tokio_stream::StreamExt;

lazy_static! {
    static ref EVENT_IN: &'static str = "event_in";
}

#[derive(Parser)]
struct Args {
    /// The program ID to subscribe to
    #[clap(
        short = 'p',
        default_value = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
    )]
    program_id: String,
    #[clap(short = 'k', default_value = "5d166540-f22e-4f66-bb70-8349844d4a0e")]
    api_key: String,
    #[clap(short = 'r', default_value = "redis://127.0.0.1/")]
    redis_url: String,
    #[clap(short = 'q', default_value = "events")]
    redis_queue: String,
}

#[tokio::main]
async fn main() {
    if let Err(e) = entrypoint().await {
        eprintln!("Error: {:?}", e);
    }
}

async fn subscribe_and_process(
    api_key: &str,
    program_id: &pubkey::Pubkey,
    redis_url: &str,
    redis_queue: &str,
) -> Result<()> {
    let redis_client =
        redis::Client::open(redis_url).map_err(|e| HeliusError::EnhancedWebsocket {
            reason: "Redis connection error".to_string(),
            message: e.to_string(),
        })?;
    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| HeliusError::EnhancedWebsocket {
            reason: "Redis connection error".to_string(),
            message: e.to_string(),
        })?;
    redis_conn
        .set::<&str, i32, ()>(&EVENT_IN, 0)
        .await
        .map_err(|e| HeliusError::EnhancedWebsocket {
            reason: format!("Failed to set event counter in Redis: {:?}", e),
            message: e.to_string(),
        })?;

    // wss://atlas-mainnet.helius-rpc.com/?api-key=<API_KEY>
    let cluster: Cluster = Cluster::MainnetBeta;
    // Uses custom ping-pong timeouts to ping every 15s and timeout after 45s of no pong
    let helius: Helius =
        Helius::new_with_ws_with_timeouts(api_key, cluster, Some(15), Some(45)).await?;

    dbg!(&helius.config.endpoints);
    dbg!(&helius.config.cluster);

    let config: RpcTransactionsConfig = RpcTransactionsConfig {
        filter: TransactionSubscribeFilter::standard(program_id),
        options: TransactionSubscribeOptions::default(),
    };

    if let Some(ws) = helius.ws() {
        let (mut stream, _unsub) = ws.transaction_subscribe(config).await?;
        println!("Stream subscribe success, {}", Utc::now());

        let mut notify_count: usize = 0;

        // There may be a timeout while waiting for the next notification from the stream.
        // This can happen if the downstream is hanged or if there are no new events within the specified duration.
        while let Some(notification) = timeout(Duration::from_secs(60), stream.next())
            .await
            .unwrap_or(None)
        {
            let notification_str = serde_json::to_string(&notification).unwrap();

            // add event counter in redis
            redis_conn
                .incr::<&str, i32, ()>(&EVENT_IN, 1)
                .await
                .map_err(|e| HeliusError::EnhancedWebsocket {
                    reason: format!("Failed to increment event counter in Redis: {:?}", e),
                    message: e.to_string(),
                })?;
            redis_conn
                .rpush::<&str, &str, ()>(redis_queue, &notification_str)
                .await
                .map_err(|e| HeliusError::EnhancedWebsocket {
                    reason: format!("Failed to send notification to Redis: {:?}", e),
                    message: e.to_string(),
                })?;
            notify_count += 1;
        }
        println!("notify counter: {}", notify_count);
    }
    Ok(())
}

async fn entrypoint() -> Result<()> {
    let args = Args::parse();
    let program_id = pubkey::Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let api_key = args.api_key.clone();
    let redis_url = args.redis_url.clone();
    let redis_queue = args.redis_queue.clone();

    dbg!(args.program_id);
    dbg!(args.api_key);
    dbg!(args.redis_url);

    loop {
        if let Err(e) = subscribe_and_process(&api_key, &program_id, &redis_url, &redis_queue).await
        {
            eprintln!("Error during stream processing: {:?}", e);
        }
        println!(
            "Stream has been exhausted, attempting to reconnect..., {}",
            Utc::now()
        );

        sleep(Duration::from_secs(1)).await;
    }
}

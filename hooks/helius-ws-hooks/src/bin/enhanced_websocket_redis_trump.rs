use clap::Parser;
use helius::error::HeliusError;
use helius::types::TransactionNotification;
use helius_ws_hooks::pumpfun_instruction_parser::{parse_notification, WrapPayload};
use lazy_static::lazy_static;
use redis::AsyncCommands;
use solana_program::pubkey;
use std::str::FromStr;

lazy_static! {
    static ref EVENT_PROCESSED: &'static str = "event_counter_trump";
}

#[derive(Parser)]
struct Args {
    #[clap(
        short = 'p',
        default_value = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
    )]
    program_id: String,
    #[clap(short = 'r', default_value = "redis://127.0.0.1/")]
    redis_url: String,
    #[clap(short = 'q', default_value = "events")]
    redis_queue: String,
    #[clap(short = 'o', default_value = "events_trump_out")]
    redis_queue_trump: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let redis_url = args.redis_url.clone();
    let program_id = pubkey::Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let redis_queue = args.redis_queue;
    let redis_queue_trump = args.redis_queue_trump;

    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    loop {
        let event: Option<(String, String)> = redis_conn.blpop(redis_queue.clone(), 0.0).await?;

        if let Err(e) = redis_conn.incr::<&str, i32, ()>(&EVENT_PROCESSED, 1).await {
            eprintln!("Failed to increment event counter in Redis: {:?}", e);
        }

        if let Some((_key, event_str)) = event {
            let notification: TransactionNotification =
                serde_json::from_str(event_str.as_str()).unwrap();
            if let Some(wrap_payload) = parse_notification(&notification, &program_id.to_string()) {
                match wrap_payload {
                    WrapPayload::Create(_, _) => {
                        // todo!()
                    }
                    WrapPayload::CreateBuy(event, _) => {
                        let json_str = serde_json::to_string(&event).unwrap();
                        if event.name.to_lowercase().contains("trump") {
                            redis_conn
                                .rpush::<&str, &str, ()>(&redis_queue_trump, &json_str)
                                .await
                                .map_err(|e| HeliusError::EnhancedWebsocket {
                                    reason: format!(
                                        "Failed to send notification to Redis: {:?}",
                                        e
                                    ),
                                    message: e.to_string(),
                                })?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

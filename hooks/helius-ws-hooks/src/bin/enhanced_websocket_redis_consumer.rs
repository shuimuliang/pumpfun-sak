use clap::Parser;
use helius::types::TransactionNotification;
use helius_ws_hooks::batch_csv_writer::{BatchCsvRecord, BatchCsvWriter};
use helius_ws_hooks::pumpfun_instruction_parser::parse_notification;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use solana_program::pubkey;
use std::str::FromStr;

lazy_static! {
    static ref EVENT_PROCESSED: &'static str = "event_processed";
}

#[derive(Parser)]
struct Args {
    #[clap(
        short = 'p',
        default_value = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
    )]
    program_id: String,
    #[clap(short = 'd', default_value = "/tmp/csv")]
    csv_dir: String,
    #[clap(short = 'r', default_value = "redis://127.0.0.1/")]
    redis_url: String,
    #[clap(short = 'q', default_value = "events")]
    redis_queue: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let csv_dir = args.csv_dir.clone();
    let redis_url = args.redis_url.clone();
    let program_id = pubkey::Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let redis_queue = args.redis_queue;

    let mut my_writer = BatchCsvWriter::new(csv_dir, 1000, 3600)?;

    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    if let Err(e) = redis_conn.set::<&str, i32, ()>(&EVENT_PROCESSED, 0).await {
        eprintln!("Failed to increment event counter in Redis: {:?}", e);
    }

    loop {
        let event: Option<(String, String)> = redis_conn.blpop(redis_queue.clone(), 0.0).await?;

        if let Err(e) = redis_conn.incr::<&str, i32, ()>(&EVENT_PROCESSED, 1).await {
            eprintln!("Failed to increment event counter in Redis: {:?}", e);
        }

        if let Some((_key, event_str)) = event {
            let notification: TransactionNotification =
                serde_json::from_str(event_str.as_str()).unwrap();
            if let Some(wrap_payload) = parse_notification(&notification, &program_id.to_string()) {
                let batch_csv_record: BatchCsvRecord = wrap_payload.into();
                my_writer.write(batch_csv_record).unwrap();
            }
        }
    }
}

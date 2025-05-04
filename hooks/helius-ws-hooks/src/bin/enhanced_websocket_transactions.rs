use clap::Parser;
use helius::{
    error::Result,
    types::{
        Cluster, RpcTransactionsConfig, TransactionNotification, TransactionSubscribeFilter,
        TransactionSubscribeOptions,
    },
    Helius,
};
use helius_ws_hooks::batch_csv_writer::{BatchCsvRecord, BatchCsvWriter};
use helius_ws_hooks::pumpfun_instruction_parser::parse_notification;
use solana_program::pubkey;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Parser)]
struct Args {
    /// The program ID to subscribe to
    #[clap(
        short = 'p',
        default_value = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
    )]
    program_id: String,
    #[clap(short = 'k', default_value = "5d166540-f22e-4f66-bb70-8349844d4a0e")]
    api_key: String,
    #[clap(short = 'd', default_value = "/tmp/csv")]
    csv_dir: String,
}

#[tokio::main]
async fn main() {
    if let Err(e) = entrypoint().await {
        eprintln!("Error: {:?}", e);
    }
}

async fn entrypoint() -> Result<()> {
    let args = Args::parse();
    let program_id = pubkey::Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let api_key = args.api_key.clone();
    let csv_dir = args.csv_dir.clone();

    dbg!(args.program_id);
    dbg!(args.api_key);
    dbg!(args.csv_dir);

    let (notification_tx, mut notification_rx) =
        mpsc::unbounded_channel::<TransactionNotification>();

    let mut my_writer = BatchCsvWriter::new(csv_dir, 1000, 3600).unwrap();

    tokio::spawn(async move {
        let cluster: Cluster = Cluster::MainnetBeta;
        let helius: Helius =
            Helius::new_with_ws_with_timeouts(&api_key, cluster, Some(15), Some(45))
                .await
                .unwrap();
        dbg!(&helius.config.endpoints);
        dbg!(&helius.config.cluster);
        let config: RpcTransactionsConfig = RpcTransactionsConfig {
            filter: TransactionSubscribeFilter::standard(&program_id),
            options: TransactionSubscribeOptions::default(),
        };

        if let Some(ws) = helius.ws() {
            let (mut stream, _unsub) = ws.transaction_subscribe(config).await.unwrap();
            while let Some(notification) = stream.next().await {
                // dbg!("{:#?}", &notification);
                notification_tx.send(notification).unwrap();
            }
        }
    });

    while let Some(notification) = notification_rx.recv().await {
        if let Some(wrap_payload) = parse_notification(&notification, &program_id.to_string()) {
            let batch_csv_record: BatchCsvRecord = wrap_payload.into();
            my_writer.write(batch_csv_record).unwrap();
        }

        // TODO: write parquet record
    }

    Ok(())
}

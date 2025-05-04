use clap::Parser;
use helius::types::TransactionNotification;
use helius_ws_hooks::pumpfun_instruction_parser::{parse_notification, WrapPayload};
use helius_ws_hooks::pumpfun_trading::orders::{
    execute_pumpfun_buy, execute_pumpfun_sell, BotOrder, TimerBotOrder,
};
use helius_ws_hooks::pumpfun_trading::{controller::Controller, utils::load_trading_config};
use lazy_static::lazy_static;
use log::{error, info, Level};
use redis::AsyncCommands;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
// use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::Duration;

lazy_static! {
    static ref EVENT_PROCESSED: &'static str = "event_looped";
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
    #[arg(short = 'b', long, default_value = "/tmp/config.toml")]
    bot_config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .filter(None, Level::Info.to_level_filter())
        .init();
    info!("Starting trading loop");

    let args = Args::parse();
    let redis_url = args.redis_url.clone();
    let program_id = Pubkey::from_str(&args.program_id).expect("Invalid program ID");
    let redis_queue = args.redis_queue;

    // Connect to Redis
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;
    if let Err(e) = redis_conn.set::<&str, i32, ()>(&EVENT_PROCESSED, 0).await {
        error!("Failed to increment event counter in Redis: {:?}", e);
    }

    let (thread2_tx, mut thread2_rx) = mpsc::channel::<BotOrder>(32);

    // thread1 will trigger a buy order in BigMint, then execute buy order in thread2
    let thread2_tx_for_t1 = thread2_tx.clone();
    let thread2_tx_for_t3 = thread2_tx;

    let (thread3_tx, mut thread3_rx) = mpsc::channel::<TimerBotOrder>(32);

    // Thread 1
    // - pop payload from thread 1 message queue(redis)
    // - Process payload data in a loop
    // - Push buy instructions to thread 2 message queue
    let mut handle1 = tokio::spawn(async move {
        // Initialize the controller with 1 bot
        let trading_config = load_trading_config(&args.bot_config);
        if trading_config.is_err() {
            error!("Failed to load trading config");
            return;
        }
        let mut controller = Controller::new(trading_config.unwrap());

        // Start the trading loop
        loop {
            let event: Result<Option<(String, String)>, _> =
                redis_conn.blpop(redis_queue.clone(), 0.0).await;
            if event.is_err() {
                error!("Failed to pop event from Redis");
                continue;
            }
            if let Err(e) = redis_conn.incr::<&str, i32, ()>(&EVENT_PROCESSED, 1).await {
                error!("Failed to increment event counter in Redis: {:?}", e);
            }

            if let Some((_key, event_str)) = event.unwrap() {
                let notification: TransactionNotification =
                    serde_json::from_str(event_str.as_str()).unwrap();
                if let Some(wrap_payload) =
                    parse_notification(&notification, &program_id.to_string())
                {
                    match wrap_payload {
                        WrapPayload::Create(event, is_paper_trade) => {
                            // to be finished
                            // let is_paper_trade = false;
                            controller.handle_create(&event, is_paper_trade);
                        }
                        WrapPayload::CreateBuy(event, is_paper_trade) => {
                            // let is_paper_trade = false;
                            let order = controller.handle_create_buy(&event, is_paper_trade);
                            if let Some(order) = order {
                                thread2_tx_for_t1.send(order).await.unwrap();
                            }
                        }
                        WrapPayload::Buy(event, is_paper_trade) => {
                            // let is_paper_trade = false;
                            controller.handle_buy(&event, is_paper_trade);
                        }
                        WrapPayload::Sell(event, is_paper_trade) => {
                            // let is_paper_trade = false;
                            controller.handle_sell(&event, is_paper_trade);
                        }
                        WrapPayload::BuySell(event, is_paper_trade) => {
                            // let is_paper_trade = false;
                            // to be finished
                            controller.handle_buy_sell(&event, is_paper_trade);
                        }
                        WrapPayload::Withdraw(event, is_paper_trade) => {
                            // let is_paper_trade = false;
                            // to be finished
                            controller.handle_withdraw(&event, is_paper_trade);
                        }
                        WrapPayload::Unknown => {}
                    }
                }
            }
        }
    });

    // Thread 2:
    // - Maintain trading instruction queue, support multithreaded execution of buy and sell subtasks
    // - Must not crash
    // - After executing buy instruction, write completion status to thread 1 message queue

    let mut handle2 = tokio::spawn(async move {
        // TODO: limit the number of concurrent buy/sell tasks
        loop {
            let order = thread2_rx.recv().await;
            if order.is_none() {
                break;
            }

            // Execute buy/sell
            // TODO: prebuild most of the buy instructions
            match order.unwrap() {
                BotOrder::Buy(ref order) => {
                    let execute_status = execute_pumpfun_buy(order).await;
                    info!("Buy order executed: {:?}, {:?}", order, execute_status);
                    // append a sellOrder to thread 3 timer with 6 seconds
                    let sell_order = order.to_sell_order();
                    thread3_tx
                        .send(TimerBotOrder::Sell(sell_order, 15))
                        .await
                        .unwrap();
                }
                BotOrder::Sell(ref order) => {
                    info!("Sell order executed: {:?}", order);
                    let execute_status = execute_pumpfun_sell(order).await;
                    if execute_status.is_err() {
                        error!("Failed to execute sell order: {:?}", order);
                    }
                }
            }
        }
    });

    // Thread 3:
    // - Timer
    // - Receive initial event, wait for specified time, then send buy/sell instruction to Thread 2

    let mut handle3 = tokio::spawn(async move {
        // Send buy/sell instruction to Thread 2
        loop {
            let order = thread3_rx.recv().await;
            match order.unwrap() {
                TimerBotOrder::Sell(order, seconds) => {
                    let thread2_tx_for_t3_clone = thread2_tx_for_t3.clone();
                    let _handle = tokio::spawn(async move {
                        // Wait for specified time
                        tokio::time::sleep(Duration::from_secs(seconds)).await;
                        let _ = thread2_tx_for_t3_clone.send(BotOrder::Sell(order)).await;
                        info!("Sell order timer finished");
                    });
                }
            }
        }
    });

    tokio::select! {
        _v1 = &mut handle1 => {
            info!("Thread 1 exited");
            handle2.abort();
            handle3.abort();
        }
        _v2 = &mut handle2 => {
            info!("Thread 2 exited");
            handle1.abort();
            handle3.abort();
        }
        _v3 = &mut handle3 => {
            info!("Thread 2 exited");
            handle1.abort();
            handle2.abort();
        }
    }

    Ok(())
}

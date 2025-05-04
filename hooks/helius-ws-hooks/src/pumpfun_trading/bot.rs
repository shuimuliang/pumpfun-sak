#![allow(dead_code)]

use crate::pumpfun_trading::utils::PriceUtil;
use log::{info, warn};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub open_pos: f64,       // The open position, 开仓数量
    pub filled_pos: f64,     // The filled position, 已成交数量
    pub last_update_ts: u64, // The timestamp of the last update
}

#[derive(Debug)]
#[repr(u8)]
pub enum MonitorType {
    BigMint,       // Represents a large minting event, 大额铸币
    PossibleParty, // Represents a potential trading party,
    ActiveTrade,   // Represents an active trade, 活跃交易
}

#[derive(Debug)]
pub struct MonitorRecord {
    pub monitor_type: MonitorType, // Type of monitor event
    pub mint_ts: u64,              // Timestamp of the minting event, 铸币时间戳
    pub last_update_ts: u64,       // Timestamp of the last update, 最后更新时间
    pub trade_count: u64,          // Number of trades, 交易次数
    pub entry_token_in_pool: f64,  // Initial token amount in the pool, 进入池中代币数量
    pub token_in_pool: f64,        // Current token amount in the pool, 当前池中代币数量
}

#[derive(Deserialize)]
pub struct BotConfig {
    pub trading: TradingConfig,
}
#[derive(Debug, Deserialize)]
pub struct TradingConfig {
    pub self_pub_key: String,            // Public key of the trader
    pub self_keypair: String,            // Wallet Keypair of the trader
    pub create_buy_trigger_lamport: u64, // Lamport amount to trigger a buy
    pub create_buy_watch_lamport: u64,   // Lamport amount to watch for buy opportunities
    pub pnl_loss_percentage: f64,        // Percentage of loss to stop trading
    pub initial_capital: f64,            // Initial capital for trading
    pub paper_trading: bool,             // Flag for paper trading mode
}

#[derive(Debug)]
pub struct Bot {
    pub config: RefCell<TradingConfig>,
    pub monitoring_token_pool: RefCell<HashMap<String, MonitorRecord>>,
    pub position_manager: RefCell<HashMap<String, Position>>,
    // TODO: shadow state for paper trading
}

impl Bot {
    pub fn new(config: TradingConfig) -> Self {
        Self {
            config: RefCell::new(config),
            monitoring_token_pool: RefCell::new(HashMap::new()),
            position_manager: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert_position(&self, mint_pk: String, token_to_buy: f64) {
        let mut position_manager = self.position_manager.borrow_mut();
        position_manager.insert(
            mint_pk,
            Position {
                open_pos: token_to_buy,
                filled_pos: 0.0,
                last_update_ts: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
    }

    pub fn insert_monitor_record(&self, mint_pk: String, now_ts: u64, curt_token_in_pool: f64) {
        let mut monitoring_token_pool = self.monitoring_token_pool.borrow_mut();
        monitoring_token_pool.insert(
            mint_pk,
            MonitorRecord {
                monitor_type: MonitorType::BigMint,
                mint_ts: now_ts,
                last_update_ts: now_ts,
                trade_count: 1,
                entry_token_in_pool: curt_token_in_pool,
                token_in_pool: curt_token_in_pool,
            },
        );
    }

    pub fn is_self_pub_key(&self, event_user_pk: &String) -> bool {
        event_user_pk == &self.config.borrow().self_pub_key
    }
    pub fn update_position(&self, mint_pk: &String, token_buy: f64) {
        // 检查buy事件的用户公钥是否与自身的公钥匹配
        // 如果匹配，尝试从 position_manager 中获取与 mint_pk 相关的持仓记录
        //   如果持仓记录存在且 open_pos 不等于 token_buy，则表示被抢先交易，需要卖出 token_buy 数量的代币，并打印警告信息。
        //   否则，更新持仓记录，将 filled_pos 设置为 token_buy，并将 open_pos 置为 0。
        // 如果 position_manager 中没有相关的持仓记录，打印警告信息并插入新的持仓记录，设置 open_pos 为 0，filled_pos 为 token_buy，并记录当前时间

        if let Some(position) = self.position_manager.borrow_mut().get_mut(mint_pk) {
            if position.open_pos != token_buy {
                warn!("WARN: we tried to buy pos {} but get {}, someone is faster, we need to sell now.", position.open_pos, token_buy);
                // todo: sell the 'token_buy' amount.
            } else {
                position.filled_pos = token_buy;
                position.open_pos = 0.0;
            }
        } else {
            warn!("WARN: we received event with our pub key but no position bookkeeping");
            self.position_manager.borrow_mut().insert(
                mint_pk.clone(),
                Position {
                    open_pos: 0.0,
                    filled_pos: token_buy,
                    last_update_ts: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            );
        }
    }

    pub fn update_monitor_record(&self, mint_pk: &String, token_buy: f64) -> f64 {
        // monitoring_token_pool 中更新指定 mint_pk 的 MonitorRecord
        // 记录新的 token_in_pool 数量、交易次数和最后更新时间。
        let mut token_delta = 0.0;
        if let Some(record) = self.monitoring_token_pool.borrow_mut().get_mut(mint_pk) {
            record.token_in_pool += token_buy;
            record.trade_count += 1;
            record.last_update_ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            token_delta = record.token_in_pool - record.entry_token_in_pool;
        }
        token_delta
    }

    pub fn update_monitor_record_on_sell(&self, mint_pk: &String, token_sell: f64) -> (f64, f64) {
        let mut prev_token_in_pool = 0.0;
        let mut token_delta = 0.0;
        if let Some(record) = self.monitoring_token_pool.borrow_mut().get_mut(mint_pk) {
            prev_token_in_pool = record.token_in_pool;
            record.token_in_pool -= token_sell;
            record.trade_count += 1;
            record.last_update_ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            token_delta = record.token_in_pool - record.entry_token_in_pool;
        }
        (prev_token_in_pool, token_delta)
    }

    pub fn handle_sell_position(
        &self,
        mint_pk: &String,
        token_sell: f64,
        prev_token_in_pool: f64,
        token_delta: f64,
        event_user_pk: &String,
    ) {
        if event_user_pk == &self.config.borrow().self_pub_key {
            self.position_manager.borrow_mut().remove(mint_pk);
            self.monitoring_token_pool.borrow_mut().remove(mint_pk);

            if let Some(position) = self.position_manager.borrow_mut().get_mut(mint_pk) {
                if position.filled_pos != token_sell {
                    warn!(
                        "WARN: sold token {} not equal to position {}.",
                        token_sell, position.filled_pos
                    );
                }

                let sell_amount = PriceUtil::calculate_sol_cost(-token_sell, prev_token_in_pool);
                self.config.borrow_mut().initial_capital -= sell_amount;
                self.position_manager.borrow_mut().remove(mint_pk);
                self.monitoring_token_pool.borrow_mut().remove(mint_pk);
                info!(
                    "INFO: Flat position, current capital:{}",
                    self.config.borrow().initial_capital
                );
            }
        } else if token_delta
            <= self
                .position_manager
                .borrow()
                .get(mint_pk)
                .unwrap()
                .filled_pos
        {
            // TODO: sell our position.
        }
    }
}

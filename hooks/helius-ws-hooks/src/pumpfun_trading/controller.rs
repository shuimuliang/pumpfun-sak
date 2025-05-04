#![allow(dead_code)]

use crate::pumpfun_instruction_parser::{
    PayloadBuy, PayloadBuySell, PayloadCreate, PayloadCreateBuy, PayloadSell, PayloadWithdraw,
};
use crate::pumpfun_trading::bot::{Bot, Position, TradingConfig};
use crate::pumpfun_trading::orders::{BotBuyOrder, BotOrder};
use crate::pumpfun_trading::utils::PriceUtil;
use log::info;
use std::cell::{Ref, RefCell};
use std::time::SystemTime;

#[derive(Debug)]
pub struct Controller {
    bot: RefCell<Bot>,
}

impl Controller {
    pub fn new(config: TradingConfig) -> Self {
        Self {
            bot: RefCell::new(Bot::new(config)),
        }
    }

    pub fn get_bot(&self) -> Ref<Bot> {
        self.bot.borrow()
    }

    pub fn handle_create(&mut self, _event: &PayloadCreate, is_paper_trade: bool) {
        if is_paper_trade {
            info!("Paper trading, skip create event");
        }
    }

    pub fn handle_create_buy(
        &mut self,
        event: &PayloadCreateBuy,
        is_paper_trade: bool,
    ) -> Option<BotOrder> {
        if is_paper_trade {
            info!("Paper trading, skip create buy event");
            return None;
        }
        let bot = self.bot.borrow();

        let create_buy_trigger_lamport: u64 = bot.config.borrow().create_buy_trigger_lamport;
        let mut _curt_capital = bot.config.borrow().initial_capital;

        let create_buy_lamport = event.max_sol_cost;
        let now_ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if bot.position_manager.borrow().iter().count() > 0 {
            return None;
        }

        // case 1: big mint, buy and hold some time
        if create_buy_lamport >= create_buy_trigger_lamport {
            let creator_sol_cost = create_buy_lamport as f64 / PriceUtil::PRICE_SCALE;
            let creator_buy_token_amount = event.amount as f64 / PriceUtil::TOKEN_SCALE;

            // 当前资金充足
            // TODO: send order with 0.2 sol, it better be an interface to choose paper trading or live trading
            // TODO: current capital as bot global variable
            if _curt_capital > 0.2 {
                _curt_capital -= 0.2;
                info!("INFO: Init new position, current capital:{}", _curt_capital);
            }

            // TODO: For paper trading, delay a configurable latency and send a fake 'Buy' event back to this loop.
            // TODO: token amount f64 round to u64
            let delta_sol = 0.001;
            let token_to_buy = PriceUtil::buy_x_sol(
                delta_sol,
                PriceUtil::INIT_SOL_IN_POOL + creator_sol_cost,
                PriceUtil::INIT_TOKEN - creator_buy_token_amount,
            );
            // For live trading, build and send transactions using non-blocking threads
            let buy_order = BotBuyOrder {
                wallet_private_base58: bot.config.borrow().self_keypair.clone(),
                mint_pk: event.mint_pk.clone(),
                amount_sol: delta_sol,
                slippage_basis_points: None,
            };

            // TODO: mint pk clone() cost
            bot.insert_position(event.mint_pk.clone(), token_to_buy);
            bot.insert_monitor_record(event.mint_pk.clone(), now_ts, creator_buy_token_amount);

            return Some(BotOrder::Buy(buy_order));
        }

        None
    }

    pub fn handle_buy(&mut self, event: &PayloadBuy, is_paper_trade: bool) {
        if is_paper_trade {
            info!("Paper trading, skip buy event");
            return;
        }

        let bot = self.bot.borrow_mut();

        let mint_pk = &event.mint_pk;
        if bot.monitoring_token_pool.borrow().contains_key(mint_pk) {
            let token_buy = event.amount as f64 / PriceUtil::TOKEN_SCALE;
            // if we get front run, we may want to clear our position.
            if bot.is_self_pub_key(&event.user_pk) {
                bot.update_position(mint_pk, token_buy);
            }

            let token_delta = bot.update_monitor_record(mint_pk, token_buy);
            if token_delta
                > bot
                    .position_manager
                    .borrow()
                    .get(mint_pk)
                    .unwrap()
                    .filled_pos
                    * 2.0
            {
                // TODO: sell token
            }
        }
    }

    pub fn handle_sell(&mut self, event: &PayloadSell, is_paper_trade: bool) {
        if is_paper_trade {
            info!("Paper trading, skip sell event");
            return;
        }

        let bot = self.bot.borrow_mut();

        let mint_pk = &event.mint_pk;
        if bot.monitoring_token_pool.borrow().contains_key(mint_pk) {
            let token_sell = event.amount as f64 / PriceUtil::TOKEN_SCALE;
            let (prev_token_in_pool, token_delta) =
                bot.update_monitor_record_on_sell(mint_pk, token_sell);

            // if we get front run, we may want to clear our position.
            if bot.is_self_pub_key(&event.user_pk) {
                bot.handle_sell_position(
                    mint_pk,
                    token_sell,
                    prev_token_in_pool,
                    token_delta,
                    &event.user_pk,
                );
            }
        }
    }

    pub fn handle_buy_sell(&mut self, _event: &PayloadBuySell, is_paper_trade: bool) {
        if is_paper_trade {
            info!("Paper trading, skip buy sell event");
        }
    }

    pub fn handle_withdraw(&mut self, _event: &PayloadWithdraw, is_paper_trade: bool) {
        if is_paper_trade {
            info!("Paper trading, skip withdraw event");
        }
    }

    // 获取当前bot开仓、成交数量
    pub fn get_position(&self, mint_pk: &str) -> Option<Position> {
        let bot = self.bot.borrow();
        let position_manager = bot.position_manager.borrow();
        position_manager.get(mint_pk).cloned()
    }
}

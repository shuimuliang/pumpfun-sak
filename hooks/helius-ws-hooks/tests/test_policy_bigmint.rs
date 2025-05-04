use helius_ws_hooks::pumpfun_trading::{bot::TradingConfig, controller::Controller};
mod helpers;
mod tests {
    use super::*;
    use crate::helpers::{
        create_big_mint_event, default_big_mint_event, default_buy_event, default_sell_event,
        DEFAULT_MINT_PK, DEFAULT_MINT_PK2, DEFAULT_SELF_PUB_KEY,
    };
    use helius_ws_hooks::pumpfun_trading::utils::PriceUtil;

    #[test]
    fn test_snipe_create_buy() {
        let config = TradingConfig {
            create_buy_trigger_lamport: 20_000_000, // 0.02 SOL
            create_buy_watch_lamport: 1_500_000_000,
            pnl_loss_percentage: 0.05,
            initial_capital: 5.0,
            self_pub_key: DEFAULT_SELF_PUB_KEY.to_string(),
            self_keypair: "random_key".to_string(),
            paper_trading: false,
        };
        let mut controller = Controller::new(config);
        let is_paper_trade = false;

        // Simulate a big mint event
        let token_amount: u64 = 57542586750788; // 57542586.750788 token
        let max_sol_cost: u64 = 1_717_000_000; // 1.717 SOL
        let event = default_big_mint_event(token_amount, max_sol_cost); // Assuming this function returns a valid event
        controller.handle_create_buy(&event, is_paper_trade);
        // dbg!(&controller);

        // Check the state after handling the event
        let position = controller.get_position(DEFAULT_MINT_PK); // Assuming this method exists
        assert!(
            position.is_some(),
            "Position should be created after handling the event"
        );
        let position = position.unwrap();
        assert!(position.open_pos > 0.0);
        assert_eq!(0.0, position.filled_pos);

        let self_buy_token_amount = 1_000_000_000;
        let buy_event =
            default_buy_event(DEFAULT_MINT_PK, DEFAULT_SELF_PUB_KEY, self_buy_token_amount);

        // snipe buy
        controller.handle_buy(&buy_event, is_paper_trade);
        // dbg!(&controller);

        // Simulate waiting for 6 seconds
        // Now, sell the position

        let amount = position.open_pos * PriceUtil::TOKEN_SCALE;
        let sell_event = default_sell_event(DEFAULT_MINT_PK, DEFAULT_SELF_PUB_KEY, amount as u64);
        controller.handle_sell(&sell_event, is_paper_trade);
        // dbg!(&controller);

        let bot = controller.get_bot();
        assert!(bot.position_manager.borrow().is_empty());
        assert!(bot.monitoring_token_pool.borrow().is_empty());
    }

    #[test]
    fn test_snipe_create_buy_only_one_token() {
        let config = TradingConfig {
            create_buy_trigger_lamport: 20_000_000, // 0.02 SOL
            create_buy_watch_lamport: 1_500_000_000,
            pnl_loss_percentage: 0.05,
            initial_capital: 5.0,
            self_pub_key: DEFAULT_SELF_PUB_KEY.to_string(),
            self_keypair: "random_key".to_string(),
            paper_trading: false,
        };
        let mut controller = Controller::new(config);
        let is_paper_trade = false;

        // Simulate a big mint event
        let token_amount: u64 = 57542586750788; // 57542586.750788 token
        let max_sol_cost: u64 = 1_717_000_000; // 1.717 SOL
        let event = default_big_mint_event(token_amount, max_sol_cost); // Assuming this function returns a valid event
        controller.handle_create_buy(&event, is_paper_trade);

        // Simulate a big mint event
        let event = create_big_mint_event(DEFAULT_MINT_PK2, token_amount, max_sol_cost); // Assuming this function returns a valid event
        controller.handle_create_buy(&event, is_paper_trade);

        assert!(controller.get_bot().position_manager.borrow().len() == 1);
    }
}

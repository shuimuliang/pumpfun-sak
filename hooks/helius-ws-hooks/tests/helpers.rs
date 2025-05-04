#![allow(dead_code)]

use helius_ws_hooks::pumpfun_instruction_parser::{PayloadBuy, PayloadCreateBuy, PayloadSell};
use helius_ws_hooks::pumpfun_trading::bot::TradingConfig;

pub const DEFAULT_SELF_PUB_KEY: &str = "8idEav1ZWKZifvbv7EavDPpmvgfdqaTNgArWSimyoiFR";
pub const DEFAULT_MINT_PK: &str = "2HFHu2og5UZD4sZ2DsTet9Gs2Yb6bHVE3eWqnKgppump";
pub const DEFAULT_MINT_PK2: &str = "3HFHu2og5UZD4sZ2DsTet9Gs2Yb6bHVE3eWqnKgppump";

pub const DEFAULT_SIGNATURE: &str =
    "37TLQ2DhpDrKQbR9eJ51NXECKxyHhAWxoxZbACdY6HgDDodvLfCYwjxBfGJuWT6qLFXr9a3CZ7J47uDb4b485kYB";
pub fn default_trading_config() -> TradingConfig {
    TradingConfig {
        create_buy_trigger_lamport: 1000000000,
        create_buy_watch_lamport: 1500000000,
        pnl_loss_percentage: 0.05,
        initial_capital: 5.0,
        self_pub_key: DEFAULT_SELF_PUB_KEY.to_string(),
        self_keypair: "random_keypair".to_string(),
        paper_trading: true,
    }
}

pub fn default_big_mint_event(amount: u64, max_sol_cost: u64) -> PayloadCreateBuy {
    PayloadCreateBuy {
        slot: 308319709,
        signature: DEFAULT_SIGNATURE.to_string(),
        mint_pk: DEFAULT_MINT_PK.to_string(),
        user_pk: "3Fhmws3fJjjwGwBiEQxRUHJvj4SqNNaxe8nWqpJmjKNk".to_string(),
        name: "GenZilla".to_string(),
        symbol: "GENZ".to_string(),
        uri: "https://ipfs.io/ipfs/Qmc4UQMH3m8rLe2qBzvXYi2FbeaFGJLvsT9hpQMqqEs3Sa".to_string(),
        bonding_curve: "f9LWJCDCmKW2F3JmxoaBTdWxfK8z7EfYhATDGr5ct6R".to_string(),
        associated_bonding_curve: "pCzEfmz3Z5hVyLVnacK29Fx5jMFgPdi7cogQrdt2ukT".to_string(),
        amount,       // 57542586750788,
        max_sol_cost, // 1717000000,
    }
}

pub fn create_big_mint_event(mint_pk: &str, amount: u64, max_sol_cost: u64) -> PayloadCreateBuy {
    PayloadCreateBuy {
        slot: 308319709,
        signature: DEFAULT_SIGNATURE.to_string(),
        mint_pk: mint_pk.to_string(),
        user_pk: "3Fhmws3fJjjwGwBiEQxRUHJvj4SqNNaxe8nWqpJmjKNk".to_string(),
        name: "GenZilla".to_string(),
        symbol: "GENZ".to_string(),
        uri: "https://ipfs.io/ipfs/Qmc4UQMH3m8rLe2qBzvXYi2FbeaFGJLvsT9hpQMqqEs3Sa".to_string(),
        bonding_curve: "f9LWJCDCmKW2F3JmxoaBTdWxfK8z7EfYhATDGr5ct6R".to_string(),
        associated_bonding_curve: "pCzEfmz3Z5hVyLVnacK29Fx5jMFgPdi7cogQrdt2ukT".to_string(),
        amount,       // 57542586750788
        max_sol_cost, // 1717000000,
    }
}

pub fn default_buy_event(mint_pk: &str, user_pk: &str, max_sol_cost: u64) -> PayloadBuy {
    PayloadBuy {
        slot: 308319709,
        signature: DEFAULT_SIGNATURE.to_string(),
        mint_pk: mint_pk.to_string(),
        user_pk: user_pk.to_string(),
        amount: 57542586750788,
        max_sol_cost, // 1717000000,j
    }
}

pub fn default_sell_event(mint_pk: &str, user_pk: &str, amount: u64) -> PayloadSell {
    PayloadSell {
        slot: 308319709,
        signature: DEFAULT_SIGNATURE.to_string(),
        mint_pk: mint_pk.to_string(),
        user_pk: user_pk.to_string(),
        amount,
        min_sol_output: 0, // 1717000000,j
    }
}

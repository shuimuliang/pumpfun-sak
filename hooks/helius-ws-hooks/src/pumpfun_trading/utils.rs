use crate::pumpfun_trading::bot::{BotConfig, TradingConfig};
use std::fs;

pub fn load_trading_config(config_path: &str) -> anyhow::Result<TradingConfig> {
    let file_content = fs::read_to_string(config_path)?;
    let config: BotConfig = toml::from_str(&file_content)?;
    Ok(config.trading)
}

pub struct PriceUtil;

impl PriceUtil {
    // 前置虚拟池为 30 枚 $SOL和 1073000191 枚代币，初始 K 值为 32190005730
    // 联合曲线定价函数为 y=1073000191 - 32190005730/(30+x)
    // x 为买入 $SOL 数量，y 为对应得到代币数量，求导可得每枚代币的价格
    pub const K: f64 = 32190005730.0; // 初始K值为32190005730
    pub const INIT_TOKEN: f64 = 1073000191.0; // 代币初始数量
    pub const INIT_SOL_IN_POOL: f64 = 30.0; // 池子初始SOL数量
    pub const FEE: f64 = 0.01;

    pub const TOKEN_SCALE: f64 = 1000000.0; // 代币数量的比例因子, 10^6
    pub const PRICE_SCALE: f64 = 1000000000.0; // SOL价格的比例因子, 10^9

    // 计算用指定数量的SOL可以买到多少代币
    pub fn calculate_sol_cost(delta_token: f64, curt_token_in_pool: f64) -> f64 {
        (Self::K / curt_token_in_pool) - (Self::K / (curt_token_in_pool + delta_token))
    }

    // 计算SOL能买到的Token数量
    pub fn buy_x_sol(delta_sol: f64, curt_sol_in_pool: f64, curt_token_in_pool: f64) -> f64 {
        let new_in_pool_token: f64 = Self::K / (curt_sol_in_pool + delta_sol);
        curt_token_in_pool - new_in_pool_token
    }

    // 计算首次购买代币所需的SOL数量
    pub fn calculate_initial_sol_cost(delta_token: f64) -> f64 {
        (Self::K / (Self::INIT_TOKEN - delta_token)) - Self::INIT_SOL_IN_POOL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_trading_config() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");

        // Write the configuration content to the temporary file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "[trading]").unwrap();
        writeln!(file, "create_buy_trigger_lamport = 1000000000").unwrap();
        writeln!(file, "create_buy_watch_lamport = 1500000000").unwrap();
        writeln!(file, "pnl_loss_percentage = 0.05").unwrap();
        writeln!(file, "initial_capital = 5").unwrap();
        writeln!(file, "self_pub_key = \"abcde\"").unwrap();
        writeln!(file, "self_keypair = \"keypair\"").unwrap();
        writeln!(file, "paper_trading = true").unwrap();

        // Load the configuration using the load_trading_config function
        let config = load_trading_config(file_path.to_str().unwrap()).unwrap();

        // Assert that the loaded configuration matches the expected values
        assert_eq!(config.create_buy_trigger_lamport, 1000000000);
        assert_eq!(config.create_buy_watch_lamport, 1500000000);
        assert_eq!(config.pnl_loss_percentage, 0.05);
        assert_eq!(config.initial_capital, 5.0);
        assert_eq!(config.self_pub_key, "abcde");
        assert_eq!(config.self_keypair, "keypair");
        assert!(config.paper_trading);
    }
    #[test]
    fn test_price_utils() {
        let create_buy_lamport: u64 = 1_000_000_000;
        let token_amount: u64 = 500_000;

        // 从链上事件获取的原始数值需要除以相应的比例因子
        let first_sol = create_buy_lamport as f64 / PriceUtil::PRICE_SCALE;
        let first_token = token_amount as f64 / PriceUtil::TOKEN_SCALE;
        // 计算当前池子中的代币数量
        let curt_token_in_pool = PriceUtil::INIT_SOL_IN_POOL - first_token;

        assert_eq!(first_sol, 1.0);
        assert_eq!(first_token, 0.5);
        assert_eq!(curt_token_in_pool, 29.5);

        // 计算买入token的数量
        let delta_sol = 0.01;
        let token_to_buy = PriceUtil::buy_x_sol(
            delta_sol,
            PriceUtil::INIT_SOL_IN_POOL,
            PriceUtil::INIT_TOKEN,
        );
        assert_eq!(token_to_buy, 357547.54781746864);

        let sol_cost = PriceUtil::calculate_sol_cost(token_to_buy, PriceUtil::INIT_TOKEN);
        assert_eq!(sol_cost, 0.009993337774819366);
    }

    #[test]
    fn test_price_utils_buy_most() {
        // 计算买入token的数量
        let delta_sol = 1.0;
        let token_to_buy = PriceUtil::buy_x_sol(
            delta_sol,
            PriceUtil::INIT_SOL_IN_POOL,
            PriceUtil::INIT_TOKEN,
        );
        assert_eq!(token_to_buy, 34612909.38709676);

        // 尽可能买入
        let delta_sol = 79.0;
        let token_to_buy = PriceUtil::buy_x_sol(
            delta_sol,
            PriceUtil::INIT_SOL_IN_POOL,
            PriceUtil::INIT_TOKEN,
        );
        assert_eq!(token_to_buy, 777679037.5137615);

        let sol_cost = PriceUtil::calculate_initial_sol_cost(token_to_buy);
        assert_eq!(sol_cost, 79.00000000000001);
    }

    #[test]
    fn test_price_utils_2nd_buy() {
        // 首次creator以 1.0 SOL买入
        let delta_sol = 1.0;
        let token_to_buy_1st = PriceUtil::buy_x_sol(
            delta_sol,
            PriceUtil::INIT_SOL_IN_POOL,
            PriceUtil::INIT_TOKEN,
        );
        // Token 数量
        assert_eq!(token_to_buy_1st, 34612909.38709676);

        // 第二次自己以 1.0 SOL买入
        let token_to_buy_2nd = PriceUtil::buy_x_sol(
            delta_sol,
            PriceUtil::INIT_SOL_IN_POOL + 1.0,
            PriceUtil::INIT_TOKEN - token_to_buy_1st,
        );
        assert_eq!(token_to_buy_2nd, 32449602.550403237);
    }
}

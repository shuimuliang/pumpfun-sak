use helius::Helius;
use log::{error, info};
use pumpfun_transaction_builder::{
    build_bundle_buy_single, build_bundle_sell_single, BuyTx, SellTx, JITO_TIP_PUBKEY_MAINNET,
};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum BotOrder {
    Buy(BotBuyOrder),
    Sell(BotSellOrder),
}

#[derive(Debug, Clone)]
pub struct BotBuyOrder {
    pub wallet_private_base58: String,
    pub mint_pk: String,
    pub amount_sol: f64,
    pub slippage_basis_points: Option<u64>,
}

impl BotBuyOrder {
    pub fn new(
        wallet_private_base58: String,
        mint_pk: String,
        amount_sol: f64,
        slippage_basis_points: Option<u64>,
    ) -> Self {
        Self {
            wallet_private_base58,
            mint_pk,
            amount_sol,
            slippage_basis_points,
        }
    }

    pub fn to_sell_order(&self) -> BotSellOrder {
        BotSellOrder {
            wallet_private_base58: self.wallet_private_base58.clone(),
            mint_pk: self.mint_pk.clone(),
            slippage_basis_points: self.slippage_basis_points,
        }
    }
}
#[derive(Debug, Clone)]
pub struct BotSellOrder {
    pub wallet_private_base58: String,
    pub mint_pk: String,
    pub slippage_basis_points: Option<u64>,
}

#[derive(Debug)]
pub enum TimerBotOrder {
    // Buy(BotBuyOrder),
    Sell(BotSellOrder, u64),
}

pub async fn execute_pumpfun_buy(buy_order: &BotBuyOrder) -> Result<(), anyhow::Error> {
    let mint_pubkey = Pubkey::from_str(&buy_order.mint_pk)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing mint pubkey: {:?}", e)))?;
    let helius_api_key = "5d166540-f22e-4f66-bb70-8349844d4a0e";

    let buy_tx = BuyTx {
        key: buy_order.wallet_private_base58.clone(),
        lamports: (buy_order.amount_sol * 1_000_000_000.0_f64) as u64,
        slippage: 500,
    };

    let api_key: &str = helius_api_key;
    let cluster = helius::types::types::Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster)?;

    let cluster_url =
        "https://mainnet.helius-rpc.com/?api-key=5d166540-f22e-4f66-bb70-8349844d4a0e";
    let jito_tip_pubkey = Pubkey::from_str(JITO_TIP_PUBKEY_MAINNET)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing jito tip pubkey 6: {:?}", e)))?;
    let txs =
        build_bundle_buy_single(&mint_pubkey, &buy_tx, &jito_tip_pubkey, Some(cluster_url)).await?;
    // if res.is_err() {
    //     error!("Error building buy bundle: {:?}", res);
    // }
    // assert!(res.is_ok());
    // let txs = res.unwrap();
    let jito_api_url = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

    match helius.send_jito_bundle(vec![txs], jito_api_url).await {
        Ok(bundle_id) => {
            info!("Transaction sent successfully: {}", bundle_id);
            let res = helius
                .get_bundle_statuses(vec![bundle_id], jito_api_url)
                .await?;
            info!("{:?}", res);
        }
        Err(e) => {
            error!("Failed to send transaction: {:?}", e);
        }
    }

    Ok(())
}

pub async fn execute_pumpfun_sell(sell_order: &BotSellOrder) -> Result<(), anyhow::Error> {
    let mint_pubkey = Pubkey::from_str(&sell_order.mint_pk)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing mint pubkey: {:?}", e)))?;
    let helius_api_key = "5d166540-f22e-4f66-bb70-8349844d4a0e";

    let sell_tx = SellTx {
        key: sell_order.wallet_private_base58.clone(),
        amount: None,
        slippage: 500,
    };

    let api_key: &str = helius_api_key;
    let cluster = helius::types::types::Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster)?;

    let close_token_ata = true;
    let cluster_url =
        "https://mainnet.helius-rpc.com/?api-key=5d166540-f22e-4f66-bb70-8349844d4a0e";
    let jito_tip_pubkey = Pubkey::from_str(JITO_TIP_PUBKEY_MAINNET)
        .map_err(|e| anyhow::Error::msg(format!("Error parsing jito tip pubkey 6: {:?}", e)))?;
    let txs = build_bundle_sell_single(
        &mint_pubkey,
        &sell_tx,
        &jito_tip_pubkey,
        Some(cluster_url),
        close_token_ata,
    )
    .await?;
    // assert!(res.is_ok());
    // let txs = res.unwrap();
    let jito_api_url = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

    match helius.send_jito_bundle(vec![txs], jito_api_url).await {
        Ok(bundle_id) => {
            info!("Transaction sent successfully: {}", bundle_id);
            let res = helius
                .get_bundle_statuses(vec![bundle_id], jito_api_url)
                .await?;
            info!("{:?}", res);
        }
        Err(e) => {
            error!("Failed to send transaction: {:?}", e);
        }
    }
    Ok(())
}

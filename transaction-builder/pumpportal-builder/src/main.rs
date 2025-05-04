use pumpportal_builder::send_local_create_tx;

#[tokio::main]
async fn main() {
    let wallet_private_base58 = "";
    let avatar_file_path = "/Desktop/cat2.png";
    let token_name = "cat5";
    let token_symbol = "cat5";
    let token_description = "cat5";
    let staked_endpoint =
        "https://mainnet.helius-rpc.com/?api-key=5d166540-f22e-4f66-bb70-8349844d4a0e";

    let init_buy_amount = 0.001;
    let slippage = 10;
    let priority_fee = 0.0005;
    let res = send_local_create_tx(
        wallet_private_base58,
        avatar_file_path,
        token_name,
        token_symbol,
        token_description,
        staked_endpoint,
        init_buy_amount,
        slippage,
        priority_fee,
    )
    .await;

    match res {
        Ok((mint_kp, tx_sig)) => {
            println!(
                "https://explorer.solana.com/tx/{}, mint_kp: {}",
                tx_sig, mint_kp
            );
        }
        Err(e) => {
            eprintln!("Failed to send transaction: {:?}", e);
        }
    };
}

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use spl_token::state::Mint;

#[allow(dead_code)]
fn get_account_info(mint_pubkey: &Pubkey) -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url);

    let response =
        client.get_account_with_commitment(mint_pubkey, CommitmentConfig::processed())?;

    if let Some(account) = response.value {
        let mint_metadata = Mint::unpack(&account.data)?;
        println!("Token decimals: {}", mint_metadata.decimals);
        println!("Token supply: {}", mint_metadata.supply);
    } else {
        println!("Account not found");
    }

    Ok(())
}

fn main() {
    let mint_pubkey = "isKotE9k7TMuXBXChAhdqipVw5o5qZEjVv9LV43i9uJ";

    let mint_pubkey: Pubkey = mint_pubkey.parse().unwrap();
    let _res = get_account_info(&mint_pubkey);
}

#[cfg(test)]
mod tests {
    use super::get_account_info;
    use solana_program::pubkey::Pubkey;

    #[ignore]
    #[test]
    fn test_account_info() {
        let mint_pubkey = "GP7gx56VH3g5mAei5n2VaDfVV6BRkJ6GiSHLhE6ypump";

        let mint_pubkey: Pubkey = mint_pubkey.parse().unwrap();
        let _res = get_account_info(&mint_pubkey);
    }
}

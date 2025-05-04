use anchor_lang::{AnchorDeserialize, Discriminator};
use helius::types::enhanced_websocket::TransactionNotification;
use pumpfun_cpi::instruction::{Buy, Create, Sell, Withdraw};
use serde::{Deserialize, Serialize};
use solana_transaction_status::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction,
    UiPartiallyDecodedInstruction,
};
use std::ops::Index;
use std::string::ToString;

use crate::batch_csv_writer::BatchCsvRecord;

#[allow(dead_code)]
enum WrapInstruction {
    Create(Create, u64, String, String, String, String, String), // slot, sig, mint_pk, user_pk, bonding_curve, associated_bonding_curve
    Buy(Buy, u64, String, String, String),                       // slot, sig, mint_pk, user_pk
    Sell(Sell, u64, String, String, String),                     // slot, sig, mint_pk, user_pk
    Withdraw(Withdraw, u64, String, String),                     // slot, sig, mint_pk
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WrapPayload {
    Create(PayloadCreate, bool),
    CreateBuy(PayloadCreateBuy, bool),
    Buy(PayloadBuy, bool),
    Sell(PayloadSell, bool),
    BuySell(PayloadBuySell, bool),
    Withdraw(PayloadWithdraw, bool),
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadCreate {
    pub slot: u64,
    pub signature: String,
    pub mint_pk: String,
    pub user_pk: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadCreateBuy {
    pub slot: u64,
    pub signature: String,
    pub mint_pk: String,
    pub user_pk: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
    pub amount: u64,
    pub max_sol_cost: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadBuy {
    pub slot: u64,
    pub signature: String,
    pub mint_pk: String,
    pub user_pk: String,
    pub amount: u64,
    pub max_sol_cost: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadSell {
    pub slot: u64,
    pub signature: String,
    pub mint_pk: String,
    pub user_pk: String,
    pub amount: u64,
    pub min_sol_output: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadBuySell {
    pub slot: u64,
    pub signature: String,
    pub mint_pk: String,
    pub user_pk: String,
    pub amount_buy: u64,
    pub max_sol_cost: u64,
    pub amount_sell: u64,
    pub min_sol_output: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadWithdraw {
    slot: u64,
    signature: String,
    mint_pk: String,
}

impl From<WrapPayload> for BatchCsvRecord {
    fn from(payload: WrapPayload) -> Self {
        match payload {
            WrapPayload::Create(payload, _is_paper_trade) => BatchCsvRecord {
                action: "create".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: Some(payload.user_pk),
                name: Some(payload.name),
                symbol: Some(payload.symbol),
                uri: Some(payload.uri),
                amount_buy: None,
                max_sol_cost: None,
                amount_sell: None,
                min_sol_output: None,
                bonding_curve: Some(payload.bonding_curve),
                associated_bonding_curve: Some(payload.associated_bonding_curve),
            },
            WrapPayload::CreateBuy(payload, _is_paper_trade) => BatchCsvRecord {
                action: "createbuy".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: Some(payload.user_pk),
                name: Some(payload.name),
                symbol: Some(payload.symbol),
                uri: Some(payload.uri),
                amount_buy: Some(payload.amount),
                max_sol_cost: Some(payload.max_sol_cost),
                amount_sell: None,
                min_sol_output: None,
                bonding_curve: Some(payload.bonding_curve),
                associated_bonding_curve: Some(payload.associated_bonding_curve),
            },
            WrapPayload::Buy(payload, _is_paper_trade) => BatchCsvRecord {
                action: "buy".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: Some(payload.user_pk),
                name: None,
                symbol: None,
                uri: None,
                amount_buy: Some(payload.amount),
                max_sol_cost: Some(payload.max_sol_cost),
                amount_sell: None,
                min_sol_output: None,
                bonding_curve: None,
                associated_bonding_curve: None,
            },
            WrapPayload::Sell(payload, _is_paper_trade) => BatchCsvRecord {
                action: "sell".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: Some(payload.user_pk),
                name: None,
                symbol: None,
                uri: None,
                amount_buy: None,
                max_sol_cost: None,
                amount_sell: Some(payload.amount),
                min_sol_output: Some(payload.min_sol_output),
                bonding_curve: None,
                associated_bonding_curve: None,
            },
            WrapPayload::BuySell(payload, _is_paper_trade) => BatchCsvRecord {
                action: "buysell".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: Some(payload.user_pk),
                name: None,
                symbol: None,
                uri: None,
                amount_buy: Some(payload.amount_buy),
                max_sol_cost: Some(payload.max_sol_cost),
                amount_sell: Some(payload.amount_sell),
                min_sol_output: Some(payload.min_sol_output),
                bonding_curve: None,
                associated_bonding_curve: None,
            },
            WrapPayload::Withdraw(payload, _is_paper_trade) => BatchCsvRecord {
                action: "withdraw".to_string(),
                slot: payload.slot,
                signature: payload.signature,
                mint_pk: Some(payload.mint_pk),
                user_pk: None,
                name: None,
                symbol: None,
                uri: None,
                amount_buy: None,
                max_sol_cost: None,
                amount_sell: None,
                min_sol_output: None,
                bonding_curve: None,
                associated_bonding_curve: None,
            },
            WrapPayload::Unknown => BatchCsvRecord {
                action: "unknown".to_string(),
                slot: 0,
                signature: "".to_string(),
                mint_pk: None,
                user_pk: None,
                name: None,
                symbol: None,
                uri: None,
                amount_buy: None,
                max_sol_cost: None,
                amount_sell: None,
                min_sol_output: None,
                bonding_curve: None,
                associated_bonding_curve: None,
            },
        }
    }
}

pub fn parse_notification(
    notification: &TransactionNotification,
    mint_pubkey: &str,
) -> Option<WrapPayload> {
    // 1. If the status is not OK, exit early.
    // 2. If the program_id matches pump.fun, decode the notification and extract instructions. The following are the possible trading types:
    // - Create ()
    // - CreateBuy ()
    // - Buy ()
    // - Sell ()
    // - BuySell ()
    // - Withdraw ()
    let transaction = &notification.transaction.transaction;

    let _transaction_meta = match &notification.transaction.meta {
        Some(meta) if meta.status.is_ok() => meta,
        _ => return None, // Early return if status is an error or meta is None
    };

    let mut wrap_instructions: Vec<WrapInstruction> = vec![];

    let slot = notification.slot;
    let signature = &notification.signature;

    // must be UiTransaction
    if let EncodedTransaction::Json(ui_transaction) = transaction {
        if let UiMessage::Parsed(ref ui_parsed_message) = ui_transaction.message {
            for ui_instruction in &ui_parsed_message.instructions {
                if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
                    ui_partially_decoded_instruction,
                )) = ui_instruction
                {
                    if ui_partially_decoded_instruction.program_id == mint_pubkey {
                        if let Some(wrap_instruction) =
                            decode_instruction(ui_partially_decoded_instruction, slot, signature)
                        {
                            wrap_instructions.push(wrap_instruction);
                        }
                    }
                }
            }
        }
    }

    build_payload(wrap_instructions)
}

fn decode_instruction(
    i: &UiPartiallyDecodedInstruction,
    slot: u64,
    signature: &str,
) -> Option<WrapInstruction> {
    if i.data.is_empty() {
        return None;
    }

    let data = bs58::decode(i.data.as_str()).into_vec();
    if data.is_err() {
        return None;
    }

    let data = data.unwrap();

    // invalid discriminator length
    if data.len() < 8 {
        return None;
    }

    let discriminator = &data[..8];

    match discriminator {
        Create::DISCRIMINATOR => {
            // _name, _symbol, _uri
            let instruction = Create::try_from_slice(&data[8..]).ok()?;
            let mint_pk = i.accounts.first()?.to_string();
            let bonding_curve = i.accounts.get(2)?.to_string();
            let associated_bonding_curve = i.accounts.get(3)?.to_string();
            let user_pk = i.accounts.get(7)?.to_string();
            Some(WrapInstruction::Create(
                instruction,
                slot,
                signature.to_string(),
                mint_pk,
                user_pk,
                bonding_curve,
                associated_bonding_curve,
            ))
        }
        Buy::DISCRIMINATOR => {
            // _amount, _max_sol_cost
            let instruction = Buy::try_from_slice(&data[8..]).ok()?;
            let mint_pk = i.accounts.get(2)?.to_string();
            let user_pk = i.accounts.get(6)?.to_string();
            Some(WrapInstruction::Buy(
                instruction,
                slot,
                signature.to_string(),
                mint_pk,
                user_pk,
            ))
        }
        Sell::DISCRIMINATOR => {
            // _amount, _min_sol_output
            let instruction = Sell::try_from_slice(&data[8..]).ok()?;
            let mint_pk = i.accounts.get(2)?.to_string();
            let user_pk = i.accounts.get(6)?.to_string();
            Some(WrapInstruction::Sell(
                instruction,
                slot,
                signature.to_string(),
                mint_pk,
                user_pk,
            ))
        }
        Withdraw::DISCRIMINATOR => {
            let instruction = Withdraw::try_from_slice(&data[8..]).ok()?;
            let mint_pk = i.accounts.get(2)?.to_string();
            Some(WrapInstruction::Withdraw(
                instruction,
                slot,
                signature.to_string(),
                mint_pk,
            ))
        }
        _ => {
            // dbg!("Unknown instruction");
            None
        }
    }
}

fn build_payload(wrap_instructions: Vec<WrapInstruction>) -> Option<WrapPayload> {
    if wrap_instructions.len() == 1 {
        let wrap_instruction = wrap_instructions.index(0);
        match wrap_instruction {
            WrapInstruction::Create(
                instruction,
                slot,
                signature,
                mint_pk,
                user_pk,
                bonding_curve,
                associated_bonding_curve,
            ) => {
                let wrap_payload = WrapPayload::Create(
                    PayloadCreate {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                        user_pk: user_pk.to_string(),
                        name: instruction._name.to_string(),
                        symbol: instruction._symbol.to_string(),
                        uri: instruction._uri.to_string(),
                        bonding_curve: bonding_curve.to_string(),
                        associated_bonding_curve: associated_bonding_curve.to_string(),
                    },
                    false,
                );
                return Some(wrap_payload);
            }
            WrapInstruction::Buy(instruction, slot, signature, mint_pk, user_pk) => {
                let wrap_payload = WrapPayload::Buy(
                    PayloadBuy {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                        user_pk: user_pk.to_string(),
                        amount: instruction._amount,
                        max_sol_cost: instruction._max_sol_cost,
                    },
                    false,
                );
                return Some(wrap_payload);
            }
            WrapInstruction::Sell(instruction, slot, signature, mint_pk, user_pk) => {
                let wrap_payload = WrapPayload::Sell(
                    PayloadSell {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                        user_pk: user_pk.to_string(),
                        amount: instruction._amount,
                        min_sol_output: instruction._min_sol_output,
                    },
                    false,
                );
                return Some(wrap_payload);
            }
            WrapInstruction::Withdraw(_instruction, slot, signature, mint_pk) => {
                let wrap_payload = WrapPayload::Withdraw(
                    PayloadWithdraw {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                    },
                    false,
                );
                return Some(wrap_payload);
            }
        }
    } else if wrap_instructions.len() == 2 {
        match (&wrap_instructions[0], &wrap_instructions[1]) {
            (
                WrapInstruction::Create(
                    create_instruction,
                    slot,
                    signature,
                    mint_pk,
                    user_pk,
                    bonding_curve,
                    associated_bonding_curve,
                ),
                WrapInstruction::Buy(buy_instruction, _, _, _, _),
            ) => {
                let wrap_payload = WrapPayload::CreateBuy(
                    PayloadCreateBuy {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                        user_pk: user_pk.to_string(),
                        name: create_instruction._name.to_string(),
                        symbol: create_instruction._symbol.to_string(),
                        uri: create_instruction._uri.to_string(),
                        bonding_curve: bonding_curve.to_string(),
                        associated_bonding_curve: associated_bonding_curve.to_string(),
                        amount: buy_instruction._amount,
                        max_sol_cost: buy_instruction._max_sol_cost,
                    },
                    false,
                );
                return Some(wrap_payload);
            }
            (
                WrapInstruction::Buy(buy_instruction, slot, signature, mint_pk, user_pk),
                WrapInstruction::Sell(sell_instruction, _, _, _, _),
            ) => {
                let wrap_payload = WrapPayload::BuySell(
                    PayloadBuySell {
                        slot: *slot,
                        signature: signature.to_string(),
                        mint_pk: mint_pk.to_string(),
                        user_pk: user_pk.to_string(),
                        amount_buy: buy_instruction._amount,
                        max_sol_cost: buy_instruction._max_sol_cost,
                        amount_sell: sell_instruction._amount,
                        min_sol_output: sell_instruction._min_sol_output,
                    },
                    false,
                );
                return Some(wrap_payload);
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::InstructionData;
    #[test]
    fn test_decode_base58_instruction() {
        let data = "AJTQ2h9DXrBkuDAB9Uhp439tdAdGa6cwZ";
        let decoded_data = bs58::decode(data).into_vec();
        assert!(decoded_data.is_ok());

        let instruction_data = decoded_data.unwrap();

        let discriminator: [u8; 8] = (&instruction_data[..8]).try_into().unwrap();
        assert_eq!(Buy::DISCRIMINATOR, discriminator);
    }

    #[test]
    fn test_instruction_create() {
        let hex_repr = "181ec828051c077707000000636368756d616e07000000636368756d616e4300000068747470733a2f2f697066732e696f2f697066732f516d546b674635396a715a5553316531725a656864665145735668366d7a764547675255765a56564c504c755132";
        let instruction_data = hex::decode(hex_repr).unwrap();

        let discriminator: [u8; 8] = (&instruction_data[..8]).try_into().unwrap();

        assert_eq!(Create::DISCRIMINATOR, discriminator);

        let create_instruction = Create::try_from_slice(&instruction_data[8..]).unwrap();
        assert_eq!("cchuman", create_instruction._name);
        assert_eq!("cchuman", create_instruction._symbol);
        assert_eq!(
            "https://ipfs.io/ipfs/QmTkgF59jqZUS1e1rZehdfQEsVh6mzvEGgRUvZVVLPLuQ2",
            create_instruction._uri
        );
    }

    #[test]
    fn test_instruction_buy() {
        //https://solscan.io/tx/4JhKQGRc2Yd3rfTjK6gbk6yDqRm1NeDHcrQKjWhpNxU98uMwvpm2ewQXNMpAupnhsd8LrEUab2vpNxkLStqKZDga

        let buy_instruction = Buy {
            _amount: 35758322578,
            _max_sol_cost: 37546238706,
        };

        let hex_repr = hex::encode(buy_instruction.data());
        assert_eq!("66063d1201daebea92b35c5308000000f222eebd08000000", hex_repr);

        let hex_repr = "66063d1201daebea92b35c5308000000f222eebd08000000";
        let instruction_data = hex::decode(hex_repr).unwrap();

        let discriminator: [u8; 8] = (&instruction_data[..8]).try_into().unwrap();

        assert_eq!(Buy::DISCRIMINATOR, discriminator);

        let buy = Buy::try_from_slice(&instruction_data[8..]).unwrap();
        assert_eq!(35758322578, buy._amount);
        assert_eq!(37546238706, buy._max_sol_cost);
    }

    #[test]
    fn test_instruction_sell() {
        // https://solscan.io/tx/vHjFfrk6iKWKtDYjXZnRXeF5EL6TMu61yWprfXPv87FQBCsoJL7wH8VDwZCBX7WqWnss1bb4inSNGatTt2W9x9D

        let sell_instruction = Sell {
            _amount: 71523000000,
            _min_sol_output: 1940508,
        };

        let hex_repr = hex::encode(sell_instruction.data());
        assert_eq!("33e685a4017f83adc05e1aa7100000001c9c1d0000000000", hex_repr);

        let hex_repr = "33e685a4017f83adc05e1aa7100000001c9c1d0000000000";
        let instruction_data = hex::decode(hex_repr).unwrap();

        let discriminator: [u8; 8] = (&instruction_data[..8]).try_into().unwrap();

        assert_eq!(Sell::DISCRIMINATOR, discriminator);

        let sell = Sell::try_from_slice(&instruction_data[8..]).unwrap();
        assert_eq!(71523000000, sell._amount);
        assert_eq!(1940508, sell._min_sol_output);
    }

    #[test]
    fn test_instruction_withdraw() {
        let withdraw_instruction = Withdraw {};

        let hex_repr = hex::encode(withdraw_instruction.data());
        assert_eq!("b712469c946da122", hex_repr);
    }

    #[test]
    fn test_payload_create() {
        let payload = PayloadCreate {
            slot: 307478317,
            signature: "2b1yDctRarzN5DTmLeYnZeBMwe3xNJxtc5mDQ8yNMJjgLJVA4VAX4AwnRynvLg7jXhHxQzH9pWy9wKGb5mwTatZD".to_string(),
            mint_pk: "8wGN8aEKcuSJ3qxjPZWsK87TGqqqRGCWp8CftPGtpump".to_string(),
            user_pk: "DibT4jmj4HnMmdwxPaQt4kkRHX5S427d2oqe2cVTnp47".to_string(),
            name: "CHESS".to_string(),
            symbol: "CHS".to_string(),
            uri: "test_uri".to_string(),
            bonding_curve: "8PkzhXamH8CkaxgGwjbi9HkkSjjA85bSbxmKhSKNbP98".to_string(),
            associated_bonding_curve: "B9LZLQv8eCAswjM8QZHeJpMS1PiLjrnDMvDZRDPVsRAA".to_string(),
        };

        let serialized_payload = serde_json::to_string(&payload).unwrap();
        let deserialized_payload: Result<PayloadCreate, _> =
            serde_json::from_str(&serialized_payload);
        assert!(deserialized_payload.is_ok());
    }

    #[test]
    fn test_payload_create_buy() {}

    #[test]
    fn test_payload_buy() {
        let payload = PayloadBuy {
            slot: 307478317,
            signature: "2b1yDctRarzN5DTmLeYnZeBMwe3xNJxtc5mDQ8yNMJjgLJVA4VAX4AwnRynvLg7jXhHxQzH9pWy9wKGb5mwTatZD".to_string(),
            mint_pk: "8wGN8aEKcuSJ3qxjPZWsK87TGqqqRGCWp8CftPGtpump".to_string(),
            user_pk: "DibT4jmj4HnMmdwxPaQt4kkRHX5S427d2oqe2cVTnp47".to_string(),
            amount: 35758322578,
            max_sol_cost: 37546238706,
        };

        let serialized_payload = serde_json::to_string(&payload).unwrap();
        let deserialized_payload: Result<PayloadBuy, _> = serde_json::from_str(&serialized_payload);
        assert!(deserialized_payload.is_ok());
    }

    #[test]
    fn test_payload_sell() {
        let payload = PayloadSell {
            slot: 307478317,
            signature: "2b1yDctRarzN5DTmLeYnZeBMwe3xNJxtc5mDQ8yNMJjgLJVA4VAX4AwnRynvLg7jXhHxQzH9pWy9wKGb5mwTatZD".to_string(),
            mint_pk: "8wGN8aEKcuSJ3qxjPZWsK87TGqqqRGCWp8CftPGtpump".to_string(),
            user_pk: "DibT4jmj4HnMmdwxPaQt4kkRHX5S427d2oqe2cVTnp47".to_string(),
            amount: 71523000000,
            min_sol_output: 1940508
        };

        let serialized_payload = serde_json::to_string(&payload).unwrap();
        let deserialized_payload: Result<PayloadSell, _> =
            serde_json::from_str(&serialized_payload);
        assert!(deserialized_payload.is_ok());
    }

    #[test]
    fn test_payload_buy_sell() {}

    #[test]
    fn test_payload_withdraw() {
        let payload = PayloadWithdraw {
            slot: 308127030,
            signature: "5xdZk2LczUzW342aqxn9J2zCnXX4f4X8dCGUx3ekrBQ9CXsbfsdsCHeSve99g6V2bitDXrmKDza7enbTyAVgR4oz".to_string(),
            mint_pk: "86go6bCbiKz5gP1MZ4ERHyJMEm9gYuZoWvvgAji6mpAV".to_string(),
        };

        let serialized_payload = serde_json::to_string(&payload).unwrap();
        let deserialized_payload: Result<PayloadWithdraw, _> =
            serde_json::from_str(&serialized_payload);
        assert!(deserialized_payload.is_ok());
    }
}

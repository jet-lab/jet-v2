use std::str::FromStr;

use jet_fixed_term::margin::state::{AutoRollConfig, BorrowAutoRollConfig, LendAutoRollConfig};
use solana_program::pubkey::Pubkey;
use wasm_bindgen::prelude::*;

use jet_instructions::fixed_term::ix::{configure_auto_roll, initialize_margin_user};

use crate::{bindings::serialization::JsSerializable, JsResult};

#[wasm_bindgen(js_name = "initializeMarginUserIx", skip_typescript)]
pub fn initialize_margin_user_js(
    margin_account: String,
    market: String,
    airspace: String,
    payer: String,
) -> JsResult {
    initialize_margin_user(
        Pubkey::from_str(&margin_account)?,
        Pubkey::from_str(&market)?,
        Pubkey::from_str(&airspace)?,
        Pubkey::from_str(&payer)?,
    )
    .to_js_default_serializer()
}

#[wasm_bindgen(js_name = "configureAutoRollLendIx", skip_typescript)]
pub fn configure_auto_roll_lend_js(
    market: String,
    margin_account: String,
    limit_price: u64,
) -> JsResult {
    configure_auto_roll(
        Pubkey::from_str(&market)?,
        Pubkey::from_str(&margin_account)?,
        AutoRollConfig::Lend(LendAutoRollConfig { limit_price }),
    )
    .to_js_default_serializer()
}

#[wasm_bindgen(js_name = "configureAutoRollBorrowIx", skip_typescript)]
pub fn configure_auto_roll_borrow_js(
    market: String,
    margin_account: String,
    roll_tenor: u64,
    limit_price: u64,
) -> JsResult {
    configure_auto_roll(
        Pubkey::from_str(&market)?,
        Pubkey::from_str(&margin_account)?,
        AutoRollConfig::Borrow(BorrowAutoRollConfig {
            roll_tenor,
            limit_price,
        }),
    )
    .to_js_default_serializer()
}

#[wasm_bindgen(typescript_custom_section)]
pub const IX_METHOD_SIGNATURES: &'static str = r#"
export function initializeMarginUserIx(
    marginAccount: string,
    market: string,
    airspace: string,
    payer: string
): WasmTransactionInstruction;
  
export function configureAutoRollLendIx(
    market: string,
    marginAccount: string,
    limitPrice: bigint
): WasmTransactionInstruction;
  
export function configureAutoRollBorrowIx(
    market: string,
    marginAccount: string,
    rollTenor: bigint,
    limitPrice: bigint
): WasmTransactionInstruction;
"#;

use std::str::FromStr;

use solana_program::pubkey::Pubkey;
use wasm_bindgen::prelude::*;

use jet_instructions::fixed_term::ix::initialize_margin_user;

use crate::{bindings::serialization::JsSerializable, JsResult};

#[wasm_bindgen(js_name = "initializeMarginUserIx")]
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

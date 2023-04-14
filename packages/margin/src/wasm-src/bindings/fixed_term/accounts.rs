use wasm_bindgen::prelude::*;

use jet_fixed_term::control::state::Market;

use crate::{bindings::serialization::JsAnchorDeserialize, JsResult};

#[wasm_bindgen(js_name = "deserializeMarketFromBuffer")]
pub fn deserialize_market(buf: &[u8]) -> JsResult {
    Market::deserialize_from_buffer(buf)
}

#[wasm_bindgen(typescript_custom_section)]
const MARKET_INFO: &'static str = r#"
/**
 * The anchor struct containing Market information
 */
export interface MarketInfo {
    versionTag: bigint
    airspace: string
    orderbookMarketState: string
    eventQueue: string
    asks: string
    bids: string
    underlyingTokenMint: string
    underlyingTokenVault: string
    ticketMint: string
    claimsMint: string
    ticketCollateralMint: string
    underlyingCollateralMint: string
    underlyingOracle: string
    ticketOracle: string
    feeVault: string
    feeDestination: string
    seed: string
    orderbookPaused: boolean
    ticketsPaused: boolean
    borrowTenor: bigint
    lendTenor: bigint
    originationFee: bigint
}
"#;

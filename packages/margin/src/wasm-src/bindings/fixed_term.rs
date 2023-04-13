use wasm_bindgen::prelude::*;

use jet_fixed_term::control::state::Market;

use super::serialization::JsAnchorDeserialize;

#[wasm_bindgen(js_name = "deserializeMarketFromBuffer")]
pub fn deserialize_market(buf: &[u8]) -> Result<JsValue, JsError> {
    Market::deserialize_from_buffer(buf)
}

#[wasm_bindgen(typescript_custom_section)]
const MARKET_INFO: &'static str = r#"
/**
 * The anchor struct containing Market information
 */
export interface MarketInfo {
    versionTag: bigint
    airspace: PublicKey
    orderbookMarketState: PublicKey
    eventQueue: PublicKey
    asks: PublicKey
    bids: PublicKey
    underlyingTokenMint: PublicKey
    underlyingTokenVault: PublicKey
    ticketMint: PublicKey
    claimsMint: PublicKey
    ticketCollateralMint: PublicKey
    underlyingCollateralMint: PublicKey
    underlyingOracle: PublicKey
    ticketOracle: PublicKey
    feeVault: PublicKey
    feeDestination: PublicKey
    seed: PublicKey
    orderbookPaused: boolean
    ticketsPaused: boolean
    borrowTenor: bigint
    lendTenor: bigint
    originationFee: bigint
}
"#;

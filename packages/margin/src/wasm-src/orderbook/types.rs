use jet_fixed_term::orderbook::state::OrderTag;
use js_sys::Uint8Array;
use solana_program::pubkey::Pubkey;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Order {
    /// Pukbey of the signer allowed to make changes to this order
    pub owner: Pubkey,
    /// Order tag used to track pdas related to this order
    pub order_tag: OrderTag,
    /// Total ticket worth of the order
    pub base_size: u64,
    /// Fixed point 32 representation of the price
    pub limit_price: u64,
}

#[wasm_bindgen]
pub struct DeprecatedOrder {
    /// Pukbey of the signer allowed to make changes to this order
    #[wasm_bindgen(getter_with_clone)]
    pub owner: Uint8Array,
    /// order tag used to track pdas related to this order
    /// 16 byte hash derived
    #[wasm_bindgen(getter_with_clone)]
    pub order_tag: Uint8Array,
    /// The orderId as found on the orderbook
    /// a u128, used for cancel order instructions
    #[wasm_bindgen(getter_with_clone)]
    pub order_id: Uint8Array,
    /// Total ticket worth of the order
    pub base_size: u64,
    /// Total underlying token worth of the order
    pub quote_size: u64,
    /// Fixed point 32 representation of the price
    pub limit_price: u64,
}

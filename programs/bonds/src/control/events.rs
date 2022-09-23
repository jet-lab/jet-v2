use anchor_lang::prelude::*;

#[event]
pub struct BondManagerInitialized {
    pub version: u64,
    pub address: Pubkey,
    pub underlying_token: Pubkey,
    pub duration: i64,
}

#[event]
pub struct OrderbookInitialized {
    pub bond_manager: Pubkey,
    pub orderbook_market_state: Pubkey,
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
}

#[event]
pub struct PositionRefreshed {
    pub borrower_account: Pubkey,
}

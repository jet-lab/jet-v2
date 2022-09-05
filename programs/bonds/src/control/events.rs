use anchor_lang::prelude::*;
use jet_margin::PositionChange;

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
pub struct ObligationMarkedDue {
    pub obligation: Pubkey,
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub balance: u64,
    pub obligation_timestamp: i64,
    pub marked_due_timestamp: i64,
}

#[event]
pub struct PositionRefreshed {
    pub orderbook_user_account: Pubkey,
    pub position_changes: Vec<PositionChange>,
}

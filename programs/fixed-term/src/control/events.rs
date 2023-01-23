use anchor_lang::prelude::*;

#[event]
pub struct MarketInitialized {
    pub version: u64,
    pub address: Pubkey,
    pub airspace: Pubkey,
    pub underlying_token_mint: Pubkey,
    pub underlying_oracle: Pubkey,
    pub ticket_oracle: Pubkey,
    pub borrow_tenor: u64,
    pub lend_tenor: u64,
}

#[event]
pub struct OrderbookInitialized {
    pub market: Pubkey,
    pub orderbook_market_state: Pubkey,
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub min_base_order_size: u64,
    pub tick_size: u64,
}

#[event]
pub struct PositionRefreshed {
    pub margin_user: Pubkey,
}

#[event]
pub struct ToggleOrderMatching {
    pub market: Pubkey,
    pub is_orderbook_paused: bool,
}

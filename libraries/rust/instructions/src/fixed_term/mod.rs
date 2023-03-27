pub mod builder;
pub mod derive;
pub mod ix;

use solana_sdk::pubkey::Pubkey;

pub use jet_fixed_term::{
    control::{instructions::InitializeMarketParams, state::Market},
    orderbook::state::{event_queue_len, orderbook_slab_len, OrderParams},
    ID as FIXED_TERM_PROGRAM,
};

pub use builder::*;

/// Admin instructions on a market always need this
pub struct MarketAdmin {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub airspace: Pubkey,
}

#[derive(Clone, Copy, Debug)]
pub struct OrderbookAddresses {
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
}

/// helpful addresses for a MarginUser account
pub struct MarginUser {
    pub address: Pubkey,
    pub claims: Pubkey,
    pub ticket_collateral: Pubkey,
    pub underlying_collateral: Pubkey,
}

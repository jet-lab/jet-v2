use anchor_lang::prelude::*;

use crate::events::OrderType;

#[event]
pub struct OrderCancelled {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub order_tag: u128,
}

#[event]
pub struct EventAdapterRegistered {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub adapter: Pubkey,
}

#[event]
pub struct OrderFilled {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub order_tag: u128,
    pub order_type: OrderType,
    pub sequence_number: u64,
    pub base_filled: u64,
    pub quote_filled: u64,
    pub fill_timestamp: i64,
    pub maturation_timestamp: i64,
}

#[event]
pub struct OrderRemoved {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub order_tag: u128,
    pub base_removed: u64,
    pub quote_removed: u64,
}

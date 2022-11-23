use anchor_lang::prelude::*;

use crate::events::OrderType;

#[event]
pub struct OrderCancelled {
    pub bond_manager: Pubkey,
    pub authority: Pubkey,
    pub order_id: u128,
}

#[event]
pub struct EventAdapterRegistered {
    pub bond_manager: Pubkey,
    pub owner: Pubkey,
    pub adapter: Pubkey,
}

#[event]
pub struct OrderFilled {
    pub bond_manager: Pubkey,
    pub authority: Pubkey,
    pub order_id: u128,
    pub order_type: OrderType,
    pub sequence_number: u64,
    pub base_filled: u64,
    pub quote_filled: u64,
    pub counterparty: Option<u128>,
    pub fill_timestamp: i64,
    pub maturation_timestamp: i64,
}

#[event]
pub struct OrderRemoved {
    pub bond_manager: Pubkey,
    pub authority: Pubkey,
    pub order_id: u128,
    pub base_removed: u64,
    pub quote_removed: u64,
    // pub remove_timestamp: i64,
}

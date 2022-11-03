use agnostic_orderbook::state::OrderSummary;
use anchor_lang::prelude::*;

#[event]
pub struct OrderCancelled {
    pub bond_manager: Pubkey,
    pub user: Pubkey,
    pub order_id: u128,
}

#[event]
pub struct LendOrder {
    pub bond_market: Pubkey,
    pub lender: Pubkey,
    pub order_summary: OrderSummary,
}

#[event]
pub struct SellTicketsOrder {
    pub bond_market: Pubkey,
    pub owner: Pubkey,
    pub order_summary: OrderSummary,
}

#[event]
pub struct EventAdapterRegistered {
    pub bond_manager: Pubkey,
    pub owner: Pubkey,
    pub adapter: Pubkey,
}

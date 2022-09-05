use agnostic_orderbook::state::OrderSummary;
use anchor_lang::prelude::*;

use super::state::{debt::Obligation, AssetKind, OrderSide};

#[event]
pub struct OrderCancelled {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub order_id: u128,
}

#[event]
pub struct OrderbookDeposit {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub amount: u64,
    pub kind: AssetKind,
}

#[event]
pub struct OrderbookUserInitialized {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct MarginBorrow {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub order_summary: OrderSummary,
}

#[event]
pub struct OrderPlaced {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub side: OrderSide,
    pub order_summary: OrderSummary,
}

#[event]
pub struct ObligationRepay {
    pub orderbook_user: Pubkey,
    pub obligation: Pubkey,
    pub repayment_amount: u64,
    pub final_balance: u64,
}

#[event]
pub struct ObligationFulfilled {
    pub obligation: Obligation,
    pub timestamp: i64,
}

#[event]
pub struct OrderbookWithdraw {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub amount: u64,
    pub kind: AssetKind,
}

#[event]
pub struct EventAdapterRegistered {
    pub bond_manager: Pubkey,
    pub orderbook_user: Pubkey,
    pub adapter: Pubkey,
}

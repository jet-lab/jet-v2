use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{event, prelude::*};

#[event]
pub struct MarginUserInitialized {
    pub bond_manager: Pubkey,
    pub borrower_account: Pubkey,
    pub margin_account: Pubkey,
    pub claims: Pubkey,
    pub collateral: Pubkey,
    pub underlying_settlement: Pubkey,
    pub ticket_settlement: Pubkey,
}

#[event]
pub struct MarginBorrow {
    pub bond_manager: Pubkey,
    pub margin_account: Pubkey,
    pub borrower_account: Pubkey,
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
    pub obligation: Pubkey,
    pub orderbook_user: Pubkey,
    pub borrower: Pubkey,
    pub timestamp: i64,
}

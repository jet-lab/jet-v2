use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{event, prelude::*};

#[event]
pub struct MarginUserInitialized {
    pub bond_manager: Pubkey,
    pub borrower_account: Pubkey,
    pub margin_account: Pubkey,
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
    pub borrower: Pubkey,
    pub timestamp: i64,
}

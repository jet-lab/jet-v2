use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{event, prelude::*};

use super::state::{MarginUser, SequenceNumber, TermLoanFlags};

#[event]
pub struct MarginUserInitialized {
    pub market: Pubkey,
    pub margin_user: Pubkey,
    pub margin_account: Pubkey,
}

#[event]
pub struct OrderPlaced {
    pub market: Pubkey,
    /// The authority placing this order, almost always the margin account
    pub authority: Pubkey,
    pub margin_user: Option<Pubkey>,
    pub order_tag: u128,
    pub order_type: OrderType,
    pub order_summary: OrderSummary,
    pub limit_price: u64,
    pub auto_stake: bool,
    pub post_only: bool,
    pub post_allowed: bool,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub enum OrderType {
    MarginBorrow,
    MarginLend,
    MarginSellTickets,
    Lend,
    SellTickets,
}

#[event]
pub struct TermLoanCreated {
    pub term_loan: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub order_tag: u128,
    pub sequence_number: u64,
    pub market: Pubkey,
    pub maturation_timestamp: i64,
    pub quote_filled: u64,
    pub base_filled: u64,
    pub flags: TermLoanFlags,
    pub fees: u64,
}

#[event]
pub struct TermLoanRepay {
    pub orderbook_user: Pubkey,
    pub term_loan: Pubkey,
    pub repayment_amount: u64,
    pub final_balance: u64,
}

#[event]
pub struct TermLoanFulfilled {
    pub term_loan: Pubkey,
    pub orderbook_user: Pubkey,
    pub borrower: Pubkey,
    pub repayment_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TermDepositCreated {
    pub term_deposit: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub order_tag: Option<u128>,
    pub sequence_number: u64,
    pub market: Pubkey,
    pub maturation_timestamp: i64,
    // Quote
    pub principal: u64,
    // Base
    pub amount: u64,
}

#[event]
pub struct DebtUpdated {
    pub margin_user: Pubkey,
    pub total_debt: u64,
    pub next_obligation_to_repay: Option<SequenceNumber>,
    pub outstanding_obligations: u64,
    pub is_past_due: bool,
}

impl DebtUpdated {
    pub fn new(user: &MarginUser) -> Self {
        Self {
            margin_user: user.derive_address(),
            total_debt: user.total_debt(),
            next_obligation_to_repay: user.next_term_loan_to_repay(),
            outstanding_obligations: user.outstanding_term_loans(),
            is_past_due: user.is_past_due(Clock::get().unwrap().unix_timestamp),
        }
    }
}

#[event]
pub struct AssetsUpdated {
    pub margin_user: Pubkey,
    pub entitled_tokens: u64,
    pub entitled_tickets: u64,
    pub ticket_collateral: u64,
    pub underlying_collateral: u64,
}

impl AssetsUpdated {
    pub fn new(user: &MarginUser) -> Result<Self> {
        Ok(Self {
            margin_user: user.derive_address(),
            entitled_tokens: user.entitled_tokens(),
            entitled_tickets: user.entitled_tickets(),
            ticket_collateral: user.ticket_collateral()?,
            underlying_collateral: user.underlying_collateral(),
        })
    }
}

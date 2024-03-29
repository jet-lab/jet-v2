use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{event, prelude::*};

use crate::tickets::state::TermDepositFlags;

use super::state::{
    BorrowAutoRollConfig, LendAutoRollConfig, MarginUser, SequenceNumber, TermLoanFlags,
};

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
    pub auto_roll: bool,
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
    /// Whether the loan is being repaid as part of an auto-roll
    pub is_auto_roll: bool,
}

#[event]
pub struct TermLoanFulfilled {
    pub term_loan: Pubkey,
    pub orderbook_user: Pubkey,
    pub borrower: Pubkey,
    pub repayment_amount: u64,
    pub timestamp: i64,
    /// Whether the loan is being fulfilled as part of an auto-roll
    pub is_auto_roll: bool,
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
    pub flags: TermDepositFlags,
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
            total_debt: user.debt().total(),
            next_obligation_to_repay: user.debt().next_term_loan_to_repay(),
            outstanding_obligations: user.debt().outstanding_term_loans(),
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
            entitled_tokens: user.assets().entitled_tokens(),
            entitled_tickets: user.assets().entitled_tickets(),
            ticket_collateral: user.assets().ticket_collateral()?,
            underlying_collateral: user.assets().underlying_collateral(),
        })
    }
}

#[event]
pub struct BorrowRollConfigUpdated {
    pub config: BorrowAutoRollConfig,
}

#[event]
pub struct LendRollConfigUpdated {
    pub config: LendAutoRollConfig,
}

#[event]
pub struct TermDepositFlagsToggled {
    pub margin_account: Pubkey,
    pub term_deposit: Pubkey,
    pub flags: TermDepositFlags,
}

#[event]
pub struct TermLoanFlagsToggled {
    pub margin_account: Pubkey,
    pub margin_user: Pubkey,
    pub term_loan: Pubkey,
    pub flags: TermLoanFlags,
}

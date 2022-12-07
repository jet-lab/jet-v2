use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{event, prelude::*};

use super::state::{Assets, Debt, ObligationFlags, ObligationSequenceNumber};

#[event]
pub struct MarginUserInitialized {
    pub bond_manager: Pubkey,
    pub borrower_account: Pubkey,
    pub margin_account: Pubkey,
    pub underlying_settlement: Pubkey,
    pub ticket_settlement: Pubkey,
}

#[event]
pub struct OrderPlaced {
    pub bond_manager: Pubkey,
    /// The authority placing this order, almost always the margin account
    pub authority: Pubkey,
    pub margin_user: Option<Pubkey>,
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
pub struct ObligationCreated {
    pub obligation: Pubkey,
    pub authority: Pubkey,
    pub order_id: Option<u128>,
    pub sequence_number: u64,
    pub bond_manager: Pubkey,
    pub maturation_timestamp: i64,
    pub quote_filled: u64,
    pub base_filled: u64,
    pub flags: ObligationFlags,
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

#[event]
pub struct DebtUpdated {
    pub margin_user: Pubkey,
    pub total_debt: u64,
    pub next_obligation_to_repay: Option<ObligationSequenceNumber>,
    pub outstanding_obligations: u64,
    pub is_past_due: bool,
}

impl From<(&Debt, Pubkey)> for DebtUpdated {
    fn from(src: (&Debt, Pubkey)) -> Self {
        let (debt, pubkey) = src;
        Self {
            margin_user: pubkey,
            total_debt: debt.total(),
            next_obligation_to_repay: debt.next_obligation_to_repay(),
            outstanding_obligations: debt.outstanding_obligations(),
            is_past_due: debt.is_past_due(),
        }
    }
}

#[event]
pub struct AssetsUpdated {
    pub margin_user: Pubkey,
    pub entitled_tokens: u64,
    pub entitled_tickets: u64,
    pub collateral: u64,
}

impl From<(&Assets, Pubkey)> for AssetsUpdated {
    fn from(src: (&Assets, Pubkey)) -> Self {
        let (assets, pubkey) = src;
        Self {
            margin_user: pubkey,
            entitled_tokens: assets.entitled_tokens,
            entitled_tickets: assets.entitled_tickets,
            collateral: assets.collateral().unwrap_or_default(),
        }
    }
}

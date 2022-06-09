use crate::{MarginPool, MarginPoolConfig};
use anchor_lang::prelude::*;

#[event]
pub struct PoolCreated {
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub loan_note_mint: Pubkey,
    pub token_mint: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub summary: MarginPoolSummary,
}

#[event]
pub struct PoolConfigured {
    pub margin_pool: Pubkey,
    pub fee_destination: Pubkey,
    pub pyth_product: Pubkey,
    pub pyth_price: Pubkey,
    pub config: MarginPoolConfig,
}

#[event]
pub struct Deposit {
    pub margin_pool: Pubkey,
    pub user: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub deposit_tokens: u64,
    pub deposit_notes: u64,
    pub summary: MarginPoolSummary,
}

#[event]
pub struct Withdraw {
    pub margin_pool: Pubkey,
    pub user: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub withdraw_tokens: u64,
    pub withdraw_notes: u64,
    pub summary: MarginPoolSummary,
}

#[event]
pub struct MarginBorrow {
    pub margin_pool: Pubkey,
    pub user: Pubkey,
    pub loan_account: Pubkey,
    pub deposit_account: Pubkey,
    pub tokens: u64,
    pub loan_notes: u64,
    pub deposit_notes: u64,
    pub summary: MarginPoolSummary,
}
#[event]
pub struct MarginRepay {
    pub margin_pool: Pubkey,
    pub user: Pubkey,
    pub loan_account: Pubkey,
    pub deposit_account: Pubkey,
    pub max_repay_tokens: u64,
    pub max_repay_notes: u64,
    pub repaid_tokens: u64,
    pub repaid_loan_notes: u64,
    pub repaid_deposit_notes: u64,
    pub summary: MarginPoolSummary,
}

#[event]
pub struct Collect {
    pub margin_pool: Pubkey,
    pub fee_notes_minted: u64,
    pub fee_tokens_claimed: u64,
    pub fee_notes_balance: u64,
    pub fee_tokens_balance: u64,
    pub summary: MarginPoolSummary,
}

/// Common fields from MarginPool for event logging.
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct MarginPoolSummary {
    pub borrowed_tokens: [u8; 24],
    pub uncollected_fees: [u8; 24],
    pub deposit_tokens: u64,
    pub deposit_notes: u64,
    pub loan_notes: u64,
    pub accrued_until: i64,
}

impl From<&MarginPool> for MarginPoolSummary {
    fn from(pool: &MarginPool) -> Self {
        MarginPoolSummary {
            borrowed_tokens: pool.borrowed_tokens,
            uncollected_fees: pool.uncollected_fees,
            deposit_tokens: pool.deposit_tokens,
            deposit_notes: pool.deposit_notes,
            loan_notes: pool.loan_notes,
            accrued_until: pool.accrued_until,
        }
    }
}

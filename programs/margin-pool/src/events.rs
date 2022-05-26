use anchor_lang::prelude::*;
use crate::{
    MarginPoolConfig, Amount,
};

#[event]
pub struct PoolCreated {
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub loan_note_mint: Pubkey,
    pub token_mint: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub accured_until: i64,
}

#[event]
pub struct PoolConfigured {
    pub margin_pool: Pubkey,
    pub authority: Pubkey,
    pub fee_destination: Pubkey,
    pub pyth_product: Pubkey, 
    pub pyth_price: Pubkey, 
    pub config: MarginPoolConfig, 
}

#[event]
pub struct Deposit {
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub depositor: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub deposit_tokens: u64,
    pub deposit_notes: u64,
    pub new_pool_deposit_tokens: u64, 
    pub new_pool_deposit_notes: u64, 
    pub new_pool_loan_notes: u64, 
    pub accrued_until: i64, 
}

#[event]
pub struct Withdraw {
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub depositor: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub withdraw_tokens: u64,
    pub withdraw_notes: u64,
    pub new_pool_deposit_tokens: u64, 
    pub new_pool_deposit_notes: u64, 
    pub new_pool_loan_notes: u64, 
    pub accrued_until: i64, 
}

#[event]
pub struct MarginBorrow {
    pub margin_account: Pubkey, 
    pub margin_pool: Pubkey, 
    pub loan_note_mint: Pubkey, 
    pub deposit_note_mint: Pubkey, 
    pub loan_account: Pubkey, 
    pub deposit_account: Pubkey, 
    pub borrow_tokens_amount: u64, 
    pub borrow_notes_amount: u64, 
    pub new_pool_deposit_tokens: u64, 
    pub new_pool_deposit_notes: u64, 
    pub new_pool_loan_notes: u64, 
    pub accrued_until: i64, 
}
#[event]
pub struct MarginRepay {
    pub margin_account: Pubkey, 
    pub margin_pool: Pubkey, 
    pub loan_note_mint: Pubkey, 
    pub deposit_note_mint: Pubkey, 
    pub loan_account: Pubkey, 
    pub deposit_account: Pubkey, 
    pub repay_tokens_amount: u64, 
    pub repay_notes_amount: u64, 
    pub new_pool_deposit_tokens: u64, 
    pub new_pool_deposit_notes: u64, 
    pub new_pool_loan_notes: u64, 
    pub accrued_until: i64, 
}

#[event]
pub struct MarginWithdraw {
    pub margin_account: Pubkey,
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub withdraw_amount: Amount,
    pub new_pool_deposit_tokens: u64, 
    pub new_pool_deposit_notes: u64, 
    pub new_pool_loan_notes: u64, 
    pub accrued_until: i64, 
}
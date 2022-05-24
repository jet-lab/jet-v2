use anchor_lang::prelude::*;
use crate::{
    MarginPoolConfig, Amount
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
    pub token_amount: u64,
}

#[event]
pub struct Withdraw {
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub depositor: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub token_amount: Amount,
}

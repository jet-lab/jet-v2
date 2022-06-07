use anchor_lang::prelude::*;

use crate::{MarginPoolParams, TokenMetadataParams};

#[event]
pub struct TokenConfigured {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub margin_pool: Pubkey,
    pub token_metadata: Pubkey,
    pub deposit_metadata: Pubkey,
    pub pyth_product: Pubkey,
    pub pyth_price: Pubkey,
    pub margin_pool_program: Pubkey,
    pub metadata_program: Pubkey,
    pub token_metadata_params: Option<TokenMetadataParams>,
    pub margin_pool_params: Option<MarginPoolParams>,
}

#[event]
pub struct AuthorityCreated {
    pub authority: Pubkey,
    pub payer: Pubkey,
}
#[event]
pub struct AdapterRegistered {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub adapter: Pubkey,
    pub metadata_account: Pubkey,
    pub metadata_program: Pubkey,
}
#[event]
pub struct TokenRegistered {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub margin_pool: Pubkey,
    pub vault: Pubkey,
    pub deposit_note_mint: Pubkey,
    pub loan_note_mint: Pubkey,
    pub token_mint: Pubkey,
    pub token_metadata: Pubkey,
    pub deposit_note_metadata: Pubkey,
    pub loan_note_metadata: Pubkey,
    pub margin_pool_program: Pubkey,
    pub metadata_program: Pubkey,
}

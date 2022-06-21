use anchor_lang::prelude::*;
use jet_metadata::{
    LiquidatorMetadata, MarginAdapterMetadata, PositionTokenMetadata, TokenMetadata,
};

#[event]
pub struct AuthorityCreated {
    pub authority: Pubkey,
    pub seed: u8,
}

#[event]
pub struct LiquidatorSet {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub liquidator_metadata: LiquidatorMetadata,
    pub metadata_account: Pubkey,
}

#[event]
pub struct AdapterRegistered {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub adapter: MarginAdapterMetadata,
    pub metadata_account: Pubkey,
}

#[event]
pub struct TokenMetadataConfigured {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub metadata_account: Pubkey,
    pub metadata: TokenMetadata,
}

#[event]
pub struct PositionTokenMetadataConfigured {
    pub requester: Pubkey,
    pub authority: Pubkey,
    pub metadata_account: Pubkey,
    pub metadata: PositionTokenMetadata,
}

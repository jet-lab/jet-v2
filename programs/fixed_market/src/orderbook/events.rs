use anchor_lang::prelude::*;

#[event]
pub struct OrderCancelled {
    pub market_manager: Pubkey,
    pub authority: Pubkey,
    pub order_id: u128,
}

#[event]
pub struct EventAdapterRegistered {
    pub market_manager: Pubkey,
    pub owner: Pubkey,
    pub adapter: Pubkey,
}

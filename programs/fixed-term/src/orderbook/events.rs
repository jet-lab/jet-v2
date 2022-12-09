use anchor_lang::prelude::*;

#[event]
pub struct OrderCancelled {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub order_id: u128,
}

#[event]
pub struct EventAdapterRegistered {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub adapter: Pubkey,
}

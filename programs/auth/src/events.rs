use anchor_lang::prelude::*;

#[event]
pub struct AuthAccountCreated {
    pub user: Pubkey,
}

#[event]
pub struct Authenticated {
    pub user: Pubkey,
}

use anchor_lang::prelude::*;

use crate::AccountPosition;

#[event]
pub struct AccountCreated {
    pub margin_account: Pubkey,
    pub owner: Pubkey,
    pub seed: u16,
}

#[event]
pub struct AccountClosed {
    pub margin_account: Pubkey,
}

#[event]
pub struct PositionRegistered {
    pub margin_account: Pubkey,
    pub authority: Pubkey,
    pub position: AccountPosition,
}

#[event]
pub struct PositionClosed {
    pub margin_account: Pubkey,
    pub authority: Pubkey,
    pub token: Pubkey,
}

#[event]
pub struct PositionBalanceUpdated {
    pub margin_account: Pubkey,
    pub position: AccountPosition,
}

#[event]
pub struct PositionTouched {
    pub position: AccountPosition,
}

#[event]
pub struct AccountingInvokeBegin {
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
}

#[event]
pub struct AccountingInvokeEnd {}

#[event]
pub struct AdapterInvokeBegin {
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
}

#[event]
pub struct AdapterInvokeEnd {}

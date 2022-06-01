use anchor_lang::prelude::*;
use crate::{
    AdapterResult
};

#[event]
pub struct AccountingInvoked {
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
    pub adapter_metadata: Pubkey,
    pub result: AdapterResult
}

#[event]
pub struct AdapterInvoked {
    pub owner: Pubkey,
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
    pub adapter_metadata: Pubkey,
    pub result: AdapterResult
}

#[event]
pub struct AccountClosed {
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub margin_account: Pubkey,
}

#[event]
pub struct PositionClosed {
    pub authority: Pubkey,
    pub receiver: Pubkey,
    pub margin_account: Pubkey,
    pub position_token_mint: Pubkey,
    pub token_account: Pubkey,
}

#[event]
pub struct AccountCreated {
    pub owner: Pubkey,
    pub margin_account: Pubkey,
}

#[event]
pub struct LiquationStarted {
    pub margin_account: Pubkey,
    pub liquidator: Pubkey,
    pub liquidator_metadata: Pubkey,
    pub liquidation_account: Pubkey,
    pub start_time: i64,
    pub min_value_change: i128,
    pub ideal_c_ratio: i128,
    pub ideal_value_liquidated: i128,
    pub fresh_collateral: i128,
    pub stale_collateral: i128,
    pub claims: i128,
}

#[event]
pub struct LiquidationEnded {
    pub authority: Pubkey,
    pub margin_account: Pubkey,
    pub liquidation_account: Pubkey,
    pub start_time: i64,
    pub value_change: i128,
    pub c_ratio_change: i128,
    pub min_value_change: i128,
}

#[event]
pub struct InvokedLiquidation {
    pub liquidator: Pubkey,
    pub liquidation: Pubkey,
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
    pub adapter_metadata: Pubkey,
    pub result: AdapterResult,
    pub fresh_collateral: i128,
    pub stale_collateral: i128,
    pub claims: i128,
}

#[event]
pub struct PositionRegistered {
    pub authority: Pubkey,
    pub margin_account: Pubkey,
    pub position_token_mint: Pubkey,
    pub metadata: Pubkey,
    pub token_account: Pubkey,
    pub kind: String,
    pub position_decimal: u8
}

#[event]
pub struct PositionBalanceUpdated {
    pub margin_account: Pubkey,
    pub token_account: Pubkey,
    pub new_balance: u64
}

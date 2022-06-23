use anchor_lang::prelude::*;

use crate::{AccountPosition, Liquidation, Valuation};

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
pub struct VerifiedHealthy {
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

#[event]
pub struct LiquidationBegun {
    pub margin_account: Pubkey,
    pub liquidator: Pubkey,
    pub liquidation: Pubkey,
    pub liquidation_data: Liquidation,
    pub valuation_summary: ValuationSummary,
}

#[event]
pub struct LiquidatorInvokeBegin {
    pub margin_account: Pubkey,
    pub adapter_program: Pubkey,
    pub liquidator: Pubkey,
}

#[event]
pub struct LiquidatorInvokeEnd {
    pub liquidation_data: Liquidation,
    pub valuation_summary: ValuationSummary,
}

#[event]
pub struct LiquidationEnded {
    pub margin_account: Pubkey,
    pub authority: Pubkey,
    pub timed_out: bool,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ValuationSummary {
    pub exposure: i128,
    pub required_collateral: i128,
    pub weighted_collateral: i128,
    pub effective_collateral: i128,
    pub available_collateral: i128,
    pub past_due: bool,
}

impl From<Valuation> for ValuationSummary {
    fn from(valuation: Valuation) -> Self {
        ValuationSummary {
            exposure: valuation.exposure.to_i128(),
            required_collateral: valuation.required_collateral.to_i128(),
            weighted_collateral: valuation.weighted_collateral.to_i128(),
            effective_collateral: valuation.effective_collateral.to_i128(),
            available_collateral: valuation.available_collateral().to_i128(),
            past_due: valuation.past_due(),
        }
    }
}

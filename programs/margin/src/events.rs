use anchor_lang::prelude::*;

use crate::{AccountPosition, Liquidation, Permissions, TokenConfigUpdate, Valuation};

event_groups! {
    PositionEvent {
        PositionRegistered,
        PositionClosed,
        PositionBalanceUpdated,
        PositionTouched
    }
}

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
pub struct PositionMetadataRefreshed {
    pub margin_account: Pubkey,
    pub position: AccountPosition,
}

#[event]
pub struct PositionBalanceUpdated {
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

#[event]
pub struct TransferPosition {
    pub source_margin_account: Pubkey,
    pub target_margin_account: Pubkey,
    pub source_token_account: Pubkey,
    pub target_token_account: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TokenConfigured {
    pub airspace: Pubkey,
    pub update: Option<TokenConfigUpdate>,
    pub mint: Pubkey,
}

#[event]
pub struct AdapterConfigured {
    pub airspace: Pubkey,
    pub adapter_program: Pubkey,
    pub is_adapter: bool,
}

#[event]
pub struct PermitConfigured {
    pub airspace: Pubkey,
    pub owner: Pubkey,
    pub permissions: Permissions,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ValuationSummary {
    pub equity: i128,
    pub liabilities: i128,
    pub required_collateral: i128,
    pub weighted_collateral: i128,
    pub effective_collateral: i128,
    pub available_collateral: i128,
    pub past_due: bool,
}

impl From<Valuation> for ValuationSummary {
    fn from(valuation: Valuation) -> Self {
        ValuationSummary {
            equity: valuation.equity.to_i128(),
            liabilities: valuation.liabilities.to_i128(),
            required_collateral: valuation.required_collateral.to_i128(),
            weighted_collateral: valuation.weighted_collateral.to_i128(),
            effective_collateral: valuation.effective_collateral.to_i128(),
            available_collateral: valuation.available_collateral().to_i128(),
            past_due: valuation.past_due(),
        }
    }
}

/// Allows you to return a single type that could actually be any of variety of events.
/// This cannot be done with traits because Box<Dyn $Name> is not possible because
/// AnchorSerialize prevents trait objects.
macro_rules! event_groups {
    ($($Name:ident{$($Variant:ident),+$(,)?})*) => {
        $(
        #[allow(clippy::enum_variant_names)]
        pub enum $Name {
            $($Variant($Variant),)+
        }

        impl $Name {
            pub fn emit(self) {
                match self {
                    $(Self::$Variant(item) => emit!(item),)+
                }
            }
        }

        $(impl From<$Variant> for $Name {
            fn from(item: $Variant) -> Self {
                Self::$Variant(item)
            }
        })+)+
    };
}
pub(crate) use event_groups;

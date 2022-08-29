// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_lang::prelude::*;

use crate::{AccountPosition, Liquidation, Valuation};

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

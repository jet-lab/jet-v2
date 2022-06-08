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

use jet_metadata::MarginAdapterMetadata;
use jet_proto_math::Number128;

use crate::adapter::{self, CompactAccountMeta, InvokeAdapter};
use crate::{
    ErrorCode, Liquidation, MarginAccount, Valuation, MAX_LIQUIDATION_COLLATERAL_RATIO,
    MAX_LIQUIDATION_C_RATIO_SLIPPAGE,
};

#[derive(Accounts)]
pub struct LiquidatorInvoke<'info> {
    /// The liquidator processing the margin account
    pub liquidator: Signer<'info>,

    /// Account to persist the state of the liquidation
    #[account(mut)]
    pub liquidation: AccountLoader<'info, Liquidation>,

    /// The margin account to proxy an action for
    #[account(mut,
              has_one = liquidation,
              has_one = liquidator)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    /// CHECK:
    pub adapter_program: AccountInfo<'info>,

    /// The metadata about the proxy program
    #[account(has_one = adapter_program)]
    pub adapter_metadata: Account<'info, MarginAdapterMetadata>,
}

pub fn liquidator_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, LiquidatorInvoke<'info>>,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<()> {
    let margin_account = &ctx.accounts.margin_account;
    let start_value = margin_account.load()?.valuation()?;

    adapter::invoke_signed(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            remaining_accounts: ctx.remaining_accounts,
        },
        account_metas,
        data,
    )?;

    update_and_verify_liquidation(
        &*ctx.accounts.margin_account.load()?,
        &mut *ctx.accounts.liquidation.load_mut()?,
        start_value,
    )
}

fn update_and_verify_liquidation(
    margin_account: &MarginAccount,
    liquidation: &mut Liquidation,
    start_value: Valuation,
) -> Result<()> {
    let end_value = margin_account.valuation()?;
    let end_c_ratio = end_value
        .c_ratio()
        .unwrap_or_else(|| Number128::from_bps(u16::MAX));
    let start_c_ratio = start_value
        .c_ratio()
        .unwrap_or_else(|| Number128::from_bps(u16::MAX));

    liquidation.value_change += end_value.net() - start_value.net(); // side effects
    liquidation.c_ratio_change += end_c_ratio - start_c_ratio; // side effects

    verify_liquidation_step_is_allowed(liquidation, end_c_ratio)
}

fn verify_liquidation_step_is_allowed(
    liquidation: &Liquidation,
    end_c_ratio: Number128,
) -> Result<()> {
    let max_c_ratio = Number128::from_bps(MAX_LIQUIDATION_COLLATERAL_RATIO);
    let max_c_ratio_slippage = Number128::from_bps(MAX_LIQUIDATION_C_RATIO_SLIPPAGE);

    if liquidation.value_change < liquidation.min_value_change {
        msg!(
            "Illegal liquidation: net loss of {} value caused by liquidation instructions which exceeds the min value change of {}",
            liquidation.value_change,
            liquidation.min_value_change
        );
        err!(ErrorCode::LiquidationLostValue)
    } else if liquidation.c_ratio_change < Number128::ZERO - max_c_ratio_slippage {
        msg!(
            "Illegal liquidation: net loss of {}% in c-ratio caused by liquidation instructions which exceeds the {} bps of allowed slippage",
            liquidation.c_ratio_change,
            max_c_ratio_slippage,
        );
        err!(ErrorCode::LiquidationUnhealthy)
    } else if end_c_ratio > max_c_ratio {
        msg!(
            "Illegal liquidation: increases collateral ratio to {} which is above the maximum {}",
            end_c_ratio,
            max_c_ratio
        );
        err!(ErrorCode::LiquidationTooHealthy)
    } else {
        Ok(())
    }
}

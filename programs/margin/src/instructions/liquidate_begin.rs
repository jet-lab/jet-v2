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

use jet_proto_math::Number128;

use crate::{
    ErrorCode, Liquidation, MarginAccount, IDEAL_LIQUIDATION_COLLATERAL_RATIO,
    MAX_LIQUIDATION_VALUE_SLIPPAGE,
};
use jet_metadata::LiquidatorMetadata;

#[derive(Accounts)]
pub struct LiquidateBegin<'info> {
    /// The account in need of liquidation
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The address paying rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The liquidator account performing the liquidation actions
    pub liquidator: Signer<'info>,

    /// The metadata describing the liquidator
    #[account(has_one = liquidator)]
    pub liquidator_metadata: Account<'info, LiquidatorMetadata>,

    /// Account to persist the state of the liquidation
    #[account(
        init,
        seeds = [
            b"liquidation",
            margin_account.key().as_ref(),
            liquidator.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Liquidation>(),
    )]
    pub liquidation: AccountLoader<'info, Liquidation>,

    system_program: Program<'info, System>,
}

pub fn liquidate_begin_handler(ctx: Context<LiquidateBegin>) -> Result<()> {
    let liquidation = &ctx.accounts.liquidation;
    let liquidator = &ctx.accounts.liquidator;
    let mut account = ctx.accounts.margin_account.load_mut()?;

    // verify the account is subject to liquidation
    account.verify_unhealthy_positions()?;

    // verify not already being liquidated
    match account.liquidation {
        liq if liq == liquidation.key() => {
            // this liquidator has already been set as the active liquidator,
            // so there is nothing to do
            unreachable!();
        }

        liq if liq == Pubkey::default() => {
            // not being liquidated, so claim it
            account.start_liquidation(liquidation.key(), liquidator.key());
        }

        _ => {
            // already claimed by some other liquidator
            return Err(ErrorCode::Liquidating.into());
        }
    }

    let valuation = account.valuation()?;
    let ideal_c_ratio = Number128::from_bps(IDEAL_LIQUIDATION_COLLATERAL_RATIO);
    let ideal_value_liquidated =
        valuation.claims() - valuation.net() / (ideal_c_ratio - Number128::ONE);

    let min_value_change = Number128::ZERO
        - Number128::from_bps(MAX_LIQUIDATION_VALUE_SLIPPAGE) * ideal_value_liquidated;

    *ctx.accounts.liquidation.load_init()? = Liquidation {
        start_time: Clock::get()?.unix_timestamp,
        value_change: Number128::ZERO,
        c_ratio_change: Number128::ZERO,
        min_value_change,
    };

    Ok(())
}

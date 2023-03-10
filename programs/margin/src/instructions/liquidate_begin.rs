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

use jet_program_common::Number128;

use crate::{
    events,
    syscall::{sys, Sys},
    ErrorCode, Liquidation, LiquidationState, MarginAccount, LIQUIDATION_MAX_TOTAL_EQUITY_LOSS_BPS,
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
        space = 8 + std::mem::size_of::<LiquidationState>(),
    )]
    pub liquidation: AccountLoader<'info, LiquidationState>,

    system_program: Program<'info, System>,
}

pub fn liquidate_begin_handler(ctx: Context<LiquidateBegin>) -> Result<()> {
    let liquidator = ctx.accounts.liquidator.key();
    let mut account = ctx.accounts.margin_account.load_mut()?;
    let timestamp = sys().unix_timestamp();

    // verify the account is subject to liquidation
    account.verify_unhealthy_positions(timestamp)?;

    // verify not already being liquidated
    match account.liquidator {
        liq if liq == liquidator => {
            // this liquidator has already been set as the active liquidator,
            // so there is nothing to do
            unreachable!();
        }

        liq if liq == Pubkey::default() => {
            // not being liquidated, so claim it
            account.start_liquidation(liquidator);
        }

        _ => {
            // already claimed by some other liquidator
            return Err(ErrorCode::Liquidating.into());
        }
    }

    let valuation = account.valuation(timestamp)?;

    let max_equity_loss =
        valuation.liabilities * Number128::from_bps(LIQUIDATION_MAX_TOTAL_EQUITY_LOSS_BPS);

    let liquidation_state = LiquidationState {
        liquidator,
        margin_account: ctx.accounts.margin_account.key(),
        state: Liquidation::new(Clock::get()?.unix_timestamp, max_equity_loss),
    };
    *ctx.accounts.liquidation.load_init()? = liquidation_state;

    emit!(events::LiquidationBegun {
        margin_account: ctx.accounts.margin_account.key(),
        liquidator,
        liquidation: ctx.accounts.liquidation.key(),
        liquidation_data: liquidation_state.state,
        valuation_summary: valuation.into(),
    });

    Ok(())
}

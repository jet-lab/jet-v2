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
use anchor_lang::AccountsClose;

use crate::{ErrorCode, Liquidation, MarginAccount, LIQUIDATION_TIMEOUT};

#[derive(Accounts)]
pub struct LiquidateEnd<'info> {
    /// If the liquidation is timed out, this can be any account
    /// If the liquidation is not timed out, this must be the liquidator, and it must be a signer
    pub authority: Signer<'info>,

    /// The account in need of liquidation
    #[account(mut, has_one = liquidation)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// Account to persist the state of the liquidation
    #[account(mut)]
    pub liquidation: AccountLoader<'info, Liquidation>,
}

pub fn liquidate_end_handler(ctx: Context<LiquidateEnd>) -> Result<()> {
    let mut account = ctx.accounts.margin_account.load_mut()?;
    let start_time = ctx.accounts.liquidation.load()?.start_time;

    if (account.liquidator != ctx.accounts.authority.key())
        && Clock::get()?.unix_timestamp - start_time < LIQUIDATION_TIMEOUT
    {
        msg!(
            "Only the liquidator may end the liquidation before the timeout of {} seconds",
            LIQUIDATION_TIMEOUT
        );
        return Err(ErrorCode::UnauthorizedLiquidator.into());
    }

    account.end_liquidation();

    ctx.accounts
        .liquidation
        .close(ctx.accounts.authority.to_account_info())?;

    Ok(())
}

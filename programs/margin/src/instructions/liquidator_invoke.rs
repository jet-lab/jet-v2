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

use crate::adapter::{self, InvokeAdapter};
use crate::{events, ErrorCode, Liquidation, MarginAccount, Valuation};

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
    data: Vec<u8>,
) -> Result<()> {
    let margin_account = &ctx.accounts.margin_account;
    let start_value = margin_account.load()?.valuation()?;

    emit!(events::LiquidatorInvokeBegin {
        margin_account: ctx.accounts.margin_account.key(),
        adapter_program: ctx.accounts.adapter_program.key(),
        liquidator: ctx.accounts.liquidator.key(),
    });

    let events = adapter::invoke(
        &InvokeAdapter {
            margin_account: &ctx.accounts.margin_account,
            adapter_program: &ctx.accounts.adapter_program,
            accounts: ctx.remaining_accounts,
            signed: true,
        },
        data,
    )?;

    for event in events {
        event.emit();
    }

    let liquidation = &mut *ctx.accounts.liquidation.load_mut()?;
    let end_value = update_and_verify_liquidation(
        &*ctx.accounts.margin_account.load()?,
        liquidation,
        start_value,
    )?;

    emit!(events::LiquidatorInvokeEnd {
        liquidation_data: *liquidation,
        valuation_summary: end_value.into(),
    });

    Ok(())
}

fn update_and_verify_liquidation(
    margin_account: &MarginAccount,
    liquidation: &mut Liquidation,
    start_value: Valuation,
) -> Result<Valuation> {
    let end_value = margin_account.valuation()?;

    *liquidation.equity_change_mut() += end_value.equity() - start_value.equity(); // side effects

    if liquidation.equity_change() < &liquidation.min_equity_change() {
        msg!(
            "Illegal liquidation: net loss of {} equity which exceeds the min value change of {}",
            liquidation.equity_change(),
            liquidation.min_equity_change()
        );
        err!(ErrorCode::LiquidationLostValue)
    } else {
        Ok(end_value)
    }
}

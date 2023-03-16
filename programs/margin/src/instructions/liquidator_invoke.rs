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

use crate::adapter::{self, InvokeAdapter};
use crate::syscall::{sys, Sys};
use crate::{
    events, AdapterConfig, ErrorCode, Liquidation, LiquidationState, MarginAccount, Valuation,
};

#[derive(Accounts)]
pub struct LiquidatorInvoke<'info> {
    /// The liquidator processing the margin account
    pub liquidator: Signer<'info>,

    /// Account to persist the state of the liquidation
    #[account(mut,
        has_one = liquidator,
        has_one = margin_account,
    )]
    pub liquidation: AccountLoader<'info, LiquidationState>,

    /// The margin account to proxy an action for
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    /// CHECK:
    pub adapter_program: AccountInfo<'info>,

    /// The metadata about the proxy program
    #[account(has_one = adapter_program,
              constraint = adapter_config.airspace == margin_account.load()?.airspace @ ErrorCode::WrongAirspace
    )]
    pub adapter_config: Account<'info, AdapterConfig>,
}

pub fn liquidator_invoke_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, LiquidatorInvoke<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    let margin_account = &ctx.accounts.margin_account;
    let start_value = margin_account.load()?.valuation(sys().unix_timestamp())?;

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

    let liquidation = &mut ctx.accounts.liquidation.load_mut()?.state;
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
    let end_value = margin_account.valuation(sys().unix_timestamp())?;

    *liquidation.equity_loss_mut() += start_value.equity - end_value.equity;

    if liquidation.equity_loss() > &liquidation.max_equity_loss() {
        msg!(
            "Illegal liquidation: net loss of {} equity which exceeds the max equity loss of {}",
            liquidation.equity_loss(),
            liquidation.max_equity_loss()
        );
        return err!(ErrorCode::LiquidationLostValue);
    }

    Ok(end_value)
}

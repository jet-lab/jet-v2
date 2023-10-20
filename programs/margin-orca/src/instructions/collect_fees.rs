// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2023 JET PROTOCOL HOLDINGS, LLC.
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

use anchor_spl::token::Token;
use orca_whirlpool::program::Whirlpool;

use crate::*;

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    #[account(mut, has_one = owner)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    pub whirlpool: Box<Account<'info, orca_whirlpool::state::Whirlpool>>,

    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position: Box<Account<'info, orca_whirlpool::state::Position>>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position_token_account: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub token_owner_account_a: UncheckedAccount<'info>,
    /// CHECK: will be validated by orca
    #[account(mut)]
    pub token_owner_account_b: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub token_vault_a: UncheckedAccount<'info>,
    /// CHECK: will be validated by orca
    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,

    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CollectFees<'info> {
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn collect_fees(&self) -> Result<()> {
        orca_whirlpool::cpi::collect_fees(CpiContext::new(
            self.orca_program.to_account_info(),
            orca_whirlpool::cpi::accounts::CollectFees {
                position: self.position.to_account_info(),
                position_token_account: self.position_token_account.to_account_info(),
                whirlpool: self.whirlpool.to_account_info(),
                token_program: self.token_program.to_account_info(),
                position_authority: self.owner.to_account_info(),
                token_owner_account_a: self.token_owner_account_a.to_account_info(),
                token_vault_a: self.token_vault_a.to_account_info(),
                token_owner_account_b: self.token_owner_account_b.to_account_info(),
                token_vault_b: self.token_vault_b.to_account_info(),
            },
        ))?;

        Ok(())
    }
}

pub fn collect_fees_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CollectFees<'info>>,
) -> Result<()> {
    // TODO: it could be useful to collect the fees directly into margin pools
    // as there's likely to be a pool for each pair that we support.
    ctx.accounts.collect_fees()?;

    // Opportunistically update the whirlpool prices as we have the account here
    let timestamp = Clock::get()?.unix_timestamp;
    ctx.accounts
        .adapter_position_metadata
        .update_whirlpool_prices(&ctx.accounts.whirlpool, timestamp);

    // We count fees as part of the collateral balance. Thus we should refresh the position
    // to reflect that those fees have been taken.
    ctx.accounts
        .adapter_position_metadata
        .update_position(&ctx.accounts.position)?;

    // Tell the margin program what the current prices are
    ctx.accounts
        .adapter_position_metadata
        .update_position_balance(&*ctx.accounts.owner.load()?, &ctx.accounts.whirlpool_config)
}

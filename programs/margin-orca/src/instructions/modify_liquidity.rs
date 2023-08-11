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
pub struct ModifyLiquidity<'info> {
    // Q: Is it fine to use an opaque Signer<'> instead?
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    #[account(mut)]
    pub whirlpool: Box<Account<'info, orca_whirlpool::state::Whirlpool>>,

    #[account(mut)]
    pub whirlpool_config: Box<Account<'info, WhirlpoolConfig>>,

    #[account(mut, has_one = owner, has_one = whirlpool_config)]
    pub adapter_position_metadata: Box<Account<'info, PositionMetadata>>,

    #[account(mut)]
    pub position: Box<Account<'info, orca_whirlpool::state::Position>>,

    /// CHECK: Will be validated by Orca
    pub position_token_account: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub token_owner_account_a: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub token_owner_account_b: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub token_vault_a: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub tick_array_lower: UncheckedAccount<'info>,

    /// CHECK: Will be validated by Orca
    #[account(mut)]
    pub tick_array_upper: UncheckedAccount<'info>,

    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ModifyLiquidity<'info> {
    #[inline(never)]
    pub fn increase_liquidity(
        &self,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<()> {
        orca_whirlpool::cpi::increase_liquidity(
            CpiContext::new(
                self.orca_program.to_account_info(),
                orca_whirlpool::cpi::accounts::ModifyLiquidity {
                    position: self.position.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    position_authority: self.owner.to_account_info(),
                    token_owner_account_a: self.token_owner_account_a.to_account_info(),
                    token_owner_account_b: self.token_owner_account_b.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    tick_array_lower: self.tick_array_lower.to_account_info(),
                    tick_array_upper: self.tick_array_upper.to_account_info(),
                },
            ),
            liquidity_amount,
            token_max_a,
            token_max_b,
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn decrease_liquidity(
        &self,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<()> {
        orca_whirlpool::cpi::decrease_liquidity(
            CpiContext::new(
                self.orca_program.to_account_info(),
                orca_whirlpool::cpi::accounts::ModifyLiquidity {
                    position: self.position.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    position_authority: self.owner.to_account_info(),
                    token_owner_account_a: self.token_owner_account_a.to_account_info(),
                    token_owner_account_b: self.token_owner_account_b.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    tick_array_lower: self.tick_array_lower.to_account_info(),
                    tick_array_upper: self.tick_array_upper.to_account_info(),
                },
            ),
            liquidity_amount,
            token_max_a,
            token_max_b,
        )?;

        Ok(())
    }
}

pub fn modify_liquidity_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ModifyLiquidity<'info>>,
    is_increase: bool,
    liquidity_amount: u128,
    token_max_a: u64,
    token_max_b: u64,
) -> Result<()> {
    if is_increase {
        ctx.accounts
            .increase_liquidity(liquidity_amount, token_max_a, token_max_b)?;
    } else {
        ctx.accounts
            .decrease_liquidity(liquidity_amount, token_max_a, token_max_b)?;
    }

    let timestamp = Clock::get()?.unix_timestamp;

    let position = &mut ctx.accounts.position;
    let whirlpool = &mut ctx.accounts.whirlpool;
    position.reload()?;
    whirlpool.reload()?;

    // Update the cache with the whirlpool and its position
    ctx.accounts
        .adapter_position_metadata
        .update_whirlpool_prices(whirlpool, timestamp);

    ctx.accounts
        .adapter_position_metadata
        .update_position(position)?;

    // Tell the margin program what the current prices are
    ctx.accounts
        .adapter_position_metadata
        .update_position_balance(&*ctx.accounts.owner.load()?, &ctx.accounts.whirlpool_config)
}

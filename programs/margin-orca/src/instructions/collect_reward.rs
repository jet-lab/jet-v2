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
pub struct CollectReward<'info> {
    /// CHECK: will be validated by orca
    pub whirlpool: UncheckedAccount<'info>,

    pub position_authority: Signer<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub position_token_account: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub reward_owner_account: UncheckedAccount<'info>,

    /// CHECK: will be validated by orca
    #[account(mut)]
    pub reward_vault: UncheckedAccount<'info>,

    pub orca_program: Program<'info, Whirlpool>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CollectReward<'info> {
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn collect_reward(&self, reward_index: u8) -> Result<()> {
        orca_whirlpool::cpi::collect_reward(
            CpiContext::new(
                self.orca_program.to_account_info(),
                orca_whirlpool::cpi::accounts::CollectReward {
                    position: self.position.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    position_authority: self.position_authority.to_account_info(),
                    reward_owner_account: self.reward_owner_account.to_account_info(),
                    reward_vault: self.reward_vault.to_account_info(),
                },
            ),
            reward_index,
        )?;

        Ok(())
    }
}

pub fn collect_reward_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CollectReward<'info>>,
    reward_index: u8,
) -> Result<()> {
    // TODO: do we want to deposit any supported tokens into a pool?

    ctx.accounts.collect_reward(reward_index)?;

    Ok(())
}

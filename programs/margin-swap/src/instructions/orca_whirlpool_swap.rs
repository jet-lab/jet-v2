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

use orca_whirlpool::program::Whirlpool;

use crate::*;

#[derive(Accounts)]
pub struct OrcaWhirlpoolSwapPoolInfo<'info> {
    /// The address of the swap program
    pub swap_program: Program<'info, Whirlpool>,

    /// The following accounts relevant to the swap pool used for the exchange
    #[account(mut)]
    pub whirlpool: Account<'info, orca_whirlpool::state::Whirlpool>,

    #[account(mut)]
    pub vault_a: AccountInfo<'info>,

    #[account(mut)]
    pub vault_b: AccountInfo<'info>,

    #[account(mut)]
    pub tick_array_0: AccountInfo<'info>,

    #[account(mut)]
    pub tick_array_1: AccountInfo<'info>,

    #[account(mut)]
    pub tick_array_2: AccountInfo<'info>,

    pub oracle: AccountInfo<'info>,
}

impl<'info> OrcaWhirlpoolSwapPoolInfo<'info> {
    /// Issue a Orca swap
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn swap(
        &self,
        authority: &AccountInfo<'info>,
        source: &AccountInfo<'info>,
        target: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        // It's safe to check only one side, if there is a mismatch, the swap ix will fail
        let source_mint = token::accessor::mint(source)?;
        let mint_a = token::accessor::mint(&self.vault_a)?;

        let (token_owner_account_a, token_owner_account_b) = if mint_a == source_mint {
            (source.to_account_info(), target.to_account_info())
        } else {
            (target.to_account_info(), source.to_account_info())
        };

        let tick_array_1 = if self.tick_array_1.owner == self.swap_program.key {
            self.tick_array_1.to_account_info()
        } else {
            self.tick_array_0.to_account_info()
        };

        let tick_array_2 = if self.tick_array_2.owner == self.swap_program.key {
            self.tick_array_2.to_account_info()
        } else {
            self.tick_array_0.to_account_info()
        };

        orca_whirlpool::cpi::swap(
            CpiContext::new(
                self.swap_program.to_account_info(),
                orca_whirlpool::cpi::accounts::Swap {
                    token_program: token_program.to_account_info(),
                    token_authority: authority.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_vault_a: self.vault_a.to_account_info(),
                    token_vault_b: self.vault_b.to_account_info(),
                    tick_array_0: self.tick_array_0.to_account_info(),
                    tick_array_1,
                    tick_array_2,
                    token_owner_account_a,
                    token_owner_account_b,
                    oracle: self.oracle.to_account_info(),
                },
            ),
            amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )?;

        Ok(())
    }
}

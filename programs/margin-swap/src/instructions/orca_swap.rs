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
pub struct OrcaWhirlpoolSwapPool<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The accounts relevant to the swap pool used for the exchange
    pub swap_info: WhirlpoolSwapInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,

    /// The address of the swap program
    pub swap_program: Program<'info, Whirlpool>,
}

impl<'info> OrcaWhirlpoolSwapPool<'info> {
    /// Issue a Orca swap
    #[inline(never)]
    fn swap(
        &self,
        source: &AccountInfo<'info>,
        target: &AccountInfo<'info>,
        withdrawal_amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        // It's safe to check only one side, if there is a mismatch, the swap ix will fail
        let source_mint = token::accessor::mint(source)?;
        let mint_a = token::accessor::mint(&self.swap_info.vault_a)?;
        let (source_vault, target_vault) = if source_mint == mint_a {
            (
                self.swap_info.vault_a.to_account_info(),
                self.swap_info.vault_b.to_account_info(),
            )
        } else {
            (
                self.swap_info.vault_b.to_account_info(),
                self.swap_info.vault_a.to_account_info(),
            )
        };
        orca_whirlpool::cpi::swap(
            CpiContext::new(
                self.swap_program.to_account_info(),
                orca_whirlpool::cpi::accounts::Swap {
                    token_program: self.token_program.to_account_info(),
                    token_authority: self.margin_account.to_account_info(),
                    whirlpool: self.swap_info.whirlpool.to_account_info(),
                    token_owner_account_a: source.to_account_info(),
                    token_vault_a: source_vault.to_account_info(),
                    token_owner_account_b: target.to_account_info(),
                    token_vault_b: target_vault.to_account_info(),
                    tick_array_0: self.swap_info.tick_array_0.to_account_info(),
                    tick_array_1: self.swap_info.tick_array_1.to_account_info(),
                    tick_array_2: self.swap_info.tick_array_2.to_account_info(),
                    oracle: self.swap_info.oracle.to_account_info(),
                },
            ),
            withdrawal_amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct WhirlpoolSwapInfo<'info> {
    /// Checked by Orca, whirlpool state
    pub whirlpool: UncheckedAccount<'info>,

    /// Checked by Orca
    pub token_authority: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub vault_a: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub vault_b: UncheckedAccount<'info>,

    /// Checked by Orca
    #[account(mut)]
    pub tick_array_0: UncheckedAccount<'info>,
    /// Checked by Orca
    #[account(mut)]
    pub tick_array_1: UncheckedAccount<'info>,
    /// Checked by Orca
    #[account(mut)]
    pub tick_array_2: UncheckedAccount<'info>,
    /// Checked by Orca
    /// Oracle is currently unused and will be enabled on subsequent updates
    pub oracle: UncheckedAccount<'info>,
}

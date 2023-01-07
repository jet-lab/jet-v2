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

use crate::*;

#[derive(Accounts)]
pub struct SaberSwapInfo<'info> {
    /// CHECK: Validated by Saber
    pub swap_pool: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    pub authority: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub vault_a: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub vault_b: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub admin_fee_a: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub admin_fee_b: AccountInfo<'info>,

    /// The address of the swap program
    pub swap_program: Program<'info, saber_stable_swap::StableSwap>,
}

impl<'info> SaberSwapInfo<'info> {
    #[inline(never)]
    pub fn swap(
        &self,
        source: &AccountInfo<'info>,
        target: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        // It's safe to check only one side, if there is a mismatch, the swap ix will fail
        let source_mint = token::accessor::mint(source)?;
        let mint_a = token::accessor::mint(&self.vault_a)?;
        let (source_vault, target_vault, fees) = if source_mint == mint_a {
            (
                self.vault_a.to_account_info(),
                self.vault_b.to_account_info(),
                self.admin_fee_b.to_account_info(),
            )
        } else {
            (
                self.vault_b.to_account_info(),
                self.vault_a.to_account_info(),
                self.admin_fee_a.to_account_info(),
            )
        };
        let swap_context = CpiContext::new(
            token_program.to_account_info(),
            saber_stable_swap::Swap {
                user: saber_stable_swap::SwapUserContext {
                    token_program: token_program.to_account_info(),
                    swap_authority: self.authority.to_account_info(),
                    user_authority: authority.to_account_info(),
                    swap: self.swap_pool.to_account_info(),
                },
                input: saber_stable_swap::SwapToken {
                    user: source.to_account_info(),
                    reserve: source_vault,
                },
                output: saber_stable_swap::SwapOutput {
                    user_token: saber_stable_swap::SwapToken {
                        user: target.to_account_info(),
                        reserve: target_vault,
                    },
                    fees,
                },
            },
        );

        saber_stable_swap::swap(swap_context, amount_in, minimum_amount_out)?;

        Ok(())
    }
}

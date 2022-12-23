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
    pub vault_into: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub vault_from: AccountInfo<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub admin_fee_destination: AccountInfo<'info>,

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
                    reserve: self.vault_into.to_account_info(),
                },
                output: saber_stable_swap::SwapOutput {
                    user_token: saber_stable_swap::SwapToken {
                        user: target.to_account_info(),
                        reserve: self.vault_from.to_account_info(),
                    },
                    fees: self.admin_fee_destination.to_account_info(),
                },
            },
        );

        saber_stable_swap::swap(swap_context, amount_in, minimum_amount_out)?;

        Ok(())
    }
}

/// A stub for saber swap, allows Anchor to generate structs for the accounts
pub fn saber_stable_swap_handler(_ctx: Context<SaberSwapInfo>) -> Result<()> {
    Err(error!(crate::ErrorCode::DisallowedDirectInstruction))
}

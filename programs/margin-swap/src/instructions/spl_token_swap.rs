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

use jet_static_program_registry::{
    orca_swap_v1, orca_swap_v2, related_programs, spl_token_swap_v2,
};

use crate::*;

// register permitted swap programs
related_programs! {
    SwapProgram {[
        spl_token_swap_v2::Spl2,
        orca_swap_v1::OrcaV1,
        orca_swap_v2::OrcaV2,
    ]}
}

#[derive(Accounts)]
pub struct SplSwapInfo<'info> {
    /// CHECK:
    pub swap_pool: AccountInfo<'info>,

    /// CHECK:
    pub authority: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_into: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_from: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,

    /// The address of the swap program
    /// CHECK:
    pub swap_program: AccountInfo<'info>,
}

impl<'info> SplSwapInfo<'info> {
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
        let swap_ix = use_client!(self.swap_program.key(), {
            client::instruction::swap(
                self.swap_program.key,
                token_program.key,
                self.swap_pool.key,
                self.authority.key,
                authority.key,
                source.key,
                self.vault_into.key,
                self.vault_from.key,
                target.key,
                self.token_mint.key,
                self.fee_account.key,
                None,
                client::instruction::Swap {
                    amount_in,
                    minimum_amount_out,
                },
            )?
        })?;

        invoke(
            &swap_ix,
            &[
                self.swap_pool.to_account_info(),
                authority.to_account_info(),
                self.authority.to_account_info(),
                source.to_account_info(),
                self.vault_into.to_account_info(),
                self.vault_from.to_account_info(),
                target.to_account_info(),
                self.token_mint.to_account_info(),
                self.fee_account.to_account_info(),
                token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

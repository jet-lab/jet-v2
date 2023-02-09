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

use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};

use crate::*;

#[derive(Accounts)]
pub struct SplSwapInfo<'info> {
    /// CHECK:
    pub swap_pool: AccountInfo<'info>,

    /// CHECK:
    pub authority: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_a: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_b: AccountInfo<'info>,

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
        // It's safe to check only one side, if there is a mismatch, the swap ix will fail
        let source_mint = token::accessor::mint(source)?;
        let mint_a = token::accessor::mint(&self.vault_a)?;
        let (source_vault, target_vault) = if source_mint == mint_a {
            (
                self.vault_a.to_account_info(),
                self.vault_b.to_account_info(),
            )
        } else {
            (
                self.vault_b.to_account_info(),
                self.vault_a.to_account_info(),
            )
        };
        let swap_ix = use_client!(self.swap_program.key(), {
            client::instruction::swap(
                self.swap_program.key,
                token_program.key,
                self.swap_pool.key,
                self.authority.key,
                authority.key,
                source.key,
                source_vault.key,
                target_vault.key,
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
                source_vault,
                target_vault,
                target.to_account_info(),
                self.token_mint.to_account_info(),
                self.fee_account.to_account_info(),
                token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

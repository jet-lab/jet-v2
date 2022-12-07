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
    pub swap_pool: UncheckedAccount<'info>,

    /// CHECK: Validated by Saber
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub vault_into: UncheckedAccount<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub vault_from: UncheckedAccount<'info>,

    /// CHECK: Validated by Saber
    #[account(mut)]
    pub admin_fee_destination: UncheckedAccount<'info>,

    /// The address of the swap program
    pub swap_program: Program<'info, saber_stable_swap::StableSwap>,
}

/// A stub for saber swap, allows Anchor to generate structs for the accounts
pub fn saber_stable_swap_handler(_ctx: Context<SaberSwapInfo>) -> Result<()> {
    Err(error!(crate::ErrorCode::DisallowedDirectInstruction))
}

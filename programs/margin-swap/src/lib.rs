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

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token;

use jet_margin::MarginAccount;
use jet_margin_pool::{
    cpi::accounts::{Deposit, Withdraw},
    program::JetMarginPool,
    Amount,
};

declare_id!("JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN");

mod instructions;
use instructions::*;

#[program]
mod jet_margin_swap {
    use super::*;

    pub fn margin_swap(
        ctx: Context<MarginSplSwap>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        margin_spl_swap_handler(ctx, amount_in, minimum_amount_out)
    }
}

#[derive(Accounts)]
pub struct MarginPoolInfo<'info> {
    /// CHECK:
    #[account(mut)]
    pub margin_pool: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub deposit_note_mint: UncheckedAccount<'info>,
}

/// Create an SPL Token Swap `Program` wrapper for validation
#[derive(Copy, Clone)]
pub struct SplTokenSwap;

impl Id for SplTokenSwap {
    fn id() -> Pubkey {
        spl_token_swap::id()
    }
}

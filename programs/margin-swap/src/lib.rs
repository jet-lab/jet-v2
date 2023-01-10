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
    ChangeKind,
};

declare_id!("JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN");

mod instructions;
use instructions::*;

#[program]
pub mod jet_margin_swap {
    use super::*;

    /// Execute a swap by withdrawing tokens from a deposit pool, swapping them for
    /// other tokens, then depositing those other tokens to another deposit pool.
    ///
    /// The instruction uses 'transit' accounts which are normally ATAs owned by the
    /// margin account. To ensure that only the tokens withdrawn are swapped and
    /// deposited, the instruction checks the balances of the transit accounts before
    /// and after an action.
    /// If either transit account has tokens before the instructions, it should still
    /// have the same tokens after the swap.
    ///
    /// **[Accounts](jet_margin_swap::accounts::MarginSplSwap) expected with margin\_spl\_swap.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The margin account being executed on. |
    /// | `source_account` | `writable` | The account with the source deposit to be exchanged from. |
    /// | `destination_account` | `writable` |  The destination account to send the deposit that is exchanged into. |
    /// | `transit_source_account` | `writable` | Temporary account for moving tokens. |
    /// | `transit_destination_account` | `writable` | Temporary account for moving tokens. |
    /// | `swap_info` | `read_only` | The accounts relevant to the swap pool used for the exchange. |
    /// | `source_margin_pool` | `read_only` | The accounts relevant to the source margin pool. |
    /// | `destination_margin_pool` | `read_only` | The accounts relevant to the destination margin pool. |
    /// | `margin_pool_program` | `read_only` | The Jet margin-pool program. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    pub fn margin_swap(
        ctx: Context<MarginSplSwap>,
        withdrawal_change_kind: ChangeKind,
        withdrawal_amount: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        margin_spl_swap_handler(
            ctx,
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        )
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

#[error_code]
pub enum ErrorCode {
    #[msg("Zero tokens have been withdrawn from a pool for the swap")]
    NoSwapTokensWithdrawn,
}

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

use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

use anchor_lang::{Id, InstructionData, ToAccountMetas};
use anchor_spl::token::Token;

use jet_margin_swap::accounts as ix_accounts;
use jet_margin_swap::instruction as ix_data;

use crate::ix_builder::MarginPoolIxBuilder;
use crate::jet_margin_pool::TokenChange;

/// Builder for creating instructions to interact with the margin swap program.
pub struct MarginSwapIxBuilder {
    /// SPL mint of the left side of the pool
    pub token_a: Pubkey,
    /// SPL mint of the right side of the pool
    pub token_b: Pubkey,
    /// The address of the swap pool
    pub swap_pool: Pubkey,
    /// The PDA of the swap pool authority
    pub swap_pool_authority: Pubkey,
    /// The mint of the swap pool notes, minted in exchange for deposits
    pub pool_mint: Pubkey,
    /// The account that accumulates transaction fees
    pub fee_account: Pubkey,
}

impl MarginSwapIxBuilder {
    /// Create a new Margin swap instruction builder
    ///
    /// # Params
    ///
    /// Refer to [MarginSwapIxBuilder] struct variables
    pub fn new(
        token_a: Pubkey,
        token_b: Pubkey,
        swap_pool: Pubkey,
        authority: Pubkey,
        pool_mint: Pubkey,
        fee_account: Pubkey,
    ) -> Self {
        Self {
            token_a,
            token_b,
            swap_pool,
            swap_pool_authority: authority,
            pool_mint,
            fee_account,
        }
    }

    /// Swap from one token to another.
    ///
    /// The source token determines the direction of the swap.
    #[allow(clippy::too_many_arguments)]
    pub fn swap(
        &self,
        margin_account: Pubkey,
        transit_src_account: Pubkey,
        transit_dst_account: Pubkey,
        source_margin_position: Pubkey,
        destination_margin_position: Pubkey,
        // swap pool token_a
        source_token_account: Pubkey,
        // swap pool token_b
        destination_token_account: Pubkey,
        swap_program: Pubkey,
        source_pool: &MarginPoolIxBuilder,
        destination_pool: &MarginPoolIxBuilder,
        change: TokenChange,
        minimum_amount_out: u64,
    ) -> Instruction {
        let accounts = ix_accounts::MarginSplSwap {
            margin_account,
            source_account: source_margin_position,
            destination_account: destination_margin_position,
            transit_source_account: transit_src_account,
            transit_destination_account: transit_dst_account,
            swap_info: ix_accounts::SwapInfo {
                swap_pool: self.swap_pool,
                authority: self.swap_pool_authority,
                vault_into: source_token_account,
                vault_from: destination_token_account,
                token_mint: self.pool_mint,
                fee_account: self.fee_account,
                swap_program,
            },
            source_margin_pool: ix_accounts::MarginPoolInfo {
                margin_pool: source_pool.address,
                vault: source_pool.vault,
                deposit_note_mint: source_pool.deposit_note_mint,
            },
            destination_margin_pool: ix_accounts::MarginPoolInfo {
                margin_pool: destination_pool.address,
                vault: destination_pool.vault,
                deposit_note_mint: destination_pool.deposit_note_mint,
            },
            margin_pool_program: jet_margin_pool::id(),
            token_program: Token::id(),
        }
        .to_account_metas(None);

        let TokenChange { kind, tokens } = change;
        Instruction {
            program_id: jet_margin_swap::id(),
            data: ix_data::MarginSwap {
                change_kind: kind,
                amount_in: tokens,
                minimum_amount_out,
            }
            .data(),
            accounts,
        }
    }
}

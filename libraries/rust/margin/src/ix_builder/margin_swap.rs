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

use std::collections::HashSet;

use anyhow::Context;
use anyhow::{bail, Result};
use solana_sdk::instruction::AccountMeta;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

use anchor_lang::{Id, InstructionData, ToAccountMetas};
use anchor_spl::token::Token;

use jet_margin_swap::instruction as ix_data;
use jet_margin_swap::{accounts as ix_accounts, SwapRouteIdentifier};
use spl_associated_token_account::get_associated_token_address;

use crate::ix_builder::MarginPoolIxBuilder;
use crate::jet_margin_pool::TokenChange;

use super::owned_position_token_account;

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
    /// The swap program
    pub swap_program: Pubkey,
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
        swap_program: Pubkey,
    ) -> Self {
        Self {
            token_a,
            token_b,
            swap_pool,
            swap_pool_authority: authority,
            pool_mint,
            fee_account,
            swap_program,
        }
    }

    /// Swap from one token to another.
    ///
    /// The source token determines the direction of the swap.
    #[allow(clippy::too_many_arguments)]
    pub fn spl_swap(
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
                withdrawal_change_kind: kind,
                withdrawal_amount: tokens,
                minimum_amount_out,
            }
            .data(),
            accounts,
        }
    }

    /// Swap from one token to another.
    ///
    /// The source token determines the direction of the swap.
    #[allow(clippy::too_many_arguments)]
    pub fn saber_swap(
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
        let accounts = ix_accounts::SaberStableSwap {
            margin_account,
            source_account: source_margin_position,
            destination_account: destination_margin_position,
            transit_source_account: transit_src_account,
            transit_destination_account: transit_dst_account,
            swap_info: ix_accounts::SaberSwapInfo {
                swap_pool: self.swap_pool,
                authority: self.swap_pool_authority,
                vault_into: source_token_account,
                vault_from: destination_token_account,
                token_mint: self.pool_mint,
                admin_fee_destination: self.fee_account,
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
            data: ix_data::SaberStableSwap {
                withdrawal_change_kind: kind,
                withdrawal_amount: tokens,
                minimum_amount_out,
            }
            .data(),
            accounts,
        }
    }
}

/// Trait to get required information from a swap pool for the [MarginSwapRouteIxBuilder]
pub trait SwapAccounts {
    /// Convert the pool to a vec of [AccountMeta]
    fn to_account_meta(&self, src_token: &Pubkey) -> Result<Vec<AccountMeta>>;
    /// Determine the pool destination token based on its source token in the swap
    fn dst_token(&self, src_token: &Pubkey) -> Result<Pubkey>;
    /// The identifier of the route
    fn route_type(&self) -> SwapRouteIdentifier;
}

/// TODO Document
///
/// TODO Do we want to refresh positions here, or separately?
/// It could make sense to expect a refresh to be separate, let's see what fits in.
pub struct MarginSwapRouteIxBuilder {
    /// The margin account creating the swap
    margin_account: Pubkey,
    /// SPL mint of the left side of the pool
    #[allow(unused)]
    src_token: Pubkey,
    /// SPL mint of the right side of the pool
    dst_token: Pubkey,
    /// Route details
    route_details: ix_data::RouteSwap,
    /// The gathered accounts of the instruction
    account_metas: Vec<AccountMeta>,
    /// The current destination token in a multi-route swap.
    /// Used to validate the swap chain
    current_route_tokens: Option<(Pubkey, Pubkey)>,
    next_route_index: usize,
    /// Whether this builder is finalized
    is_finalized: bool,
    /// Whether the next swap should be part of a multi route
    expects_multi_route: bool,
    /// SPL token accounts used, so the caller can create ATAs
    spl_token_accounts: HashSet<Pubkey>,
    /// Pool deposit notes used, so the caller can create their accounts if necessary
    pool_note_mints: HashSet<Pubkey>,
}

impl MarginSwapRouteIxBuilder {
    /// Create a new builder for a margin swap.
    /// The swap can have up to 3 steps, e.g. JET > USDC > SOL > mSOL, where each step is a leg.
    ///
    /// To get a transaction, call `finalize()`, then get the instruction via `get_instruction()`.
    pub fn new(
        margin_account: Pubkey,
        src_token: Pubkey,
        dst_token: Pubkey,
        withdrawal_change: TokenChange,
        minimum_amount_out: u64,
    ) -> Self {
        let TokenChange { kind, tokens } = withdrawal_change;
        let src_pool = MarginPoolIxBuilder::new(src_token);
        let dst_pool = MarginPoolIxBuilder::new(dst_token);

        let mut spl_token_accounts = HashSet::with_capacity(4);
        let mut pool_note_mints = HashSet::with_capacity(4);

        let (source_account, _) =
            owned_position_token_account(&margin_account, &src_pool.deposit_note_mint);
        let (destination_account, _) =
            owned_position_token_account(&margin_account, &dst_pool.deposit_note_mint);
        let transit_source_account = get_associated_token_address(&margin_account, &src_token);
        let transit_destination_account = get_associated_token_address(&margin_account, &dst_token);

        // Track accounts that should exist
        pool_note_mints.insert(src_pool.deposit_note_mint);
        pool_note_mints.insert(dst_pool.deposit_note_mint);
        spl_token_accounts.insert(src_token);
        spl_token_accounts.insert(dst_token);
        let account_metas = ix_accounts::RouteSwap {
            margin_account,
            source_account,
            destination_account,
            transit_source_account,
            transit_destination_account,
            source_margin_pool: ix_accounts::MarginPoolInfo {
                margin_pool: src_pool.address,
                vault: src_pool.vault,
                deposit_note_mint: src_pool.deposit_note_mint,
            },
            destination_margin_pool: ix_accounts::MarginPoolInfo {
                margin_pool: dst_pool.address,
                vault: dst_pool.vault,
                deposit_note_mint: dst_pool.deposit_note_mint,
            },
            margin_pool_program: jet_margin_pool::id(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None);
        Self {
            margin_account,
            src_token,
            dst_token,
            route_details: ix_data::RouteSwap {
                withdrawal_change_kind: kind,
                withdrawal_amount: tokens,
                minimum_amount_out,
                swap_routes: [Default::default(); 3],
            },
            account_metas,
            current_route_tokens: None,
            next_route_index: 0,
            is_finalized: false,
            expects_multi_route: false,
            spl_token_accounts,
            pool_note_mints,
        }
    }

    /// Add
    pub fn add_swap_route<T: SwapAccounts>(
        &mut self,
        pool: &T,
        src_token: &Pubkey,
        swap_split: u8,
    ) -> anyhow::Result<()> {
        // Check the swap split early
        if swap_split > 90 {
            bail!("Invalid swap split, must be <= 90");
        }
        // TODO: check if this is the second+ leg of a multi route, and add pool accounts for the previous
        // Check that source token is valid
        let dst_token = pool.dst_token(src_token)?;
        // Run common checks
        self.verify_addition(src_token, &dst_token, swap_split)?;

        // Add a margin pool from the previous swap if next_route > 0
        if self.next_route_index > 0 && !self.expects_multi_route {
            // It depends on whether this is a multi-hop or not.
            let pool = MarginPoolIxBuilder::new(*src_token);
            let mut pool_accounts = ix_accounts::MarginPoolInfo {
                margin_pool: pool.address,
                vault: pool.vault,
                deposit_note_mint: pool.deposit_note_mint,
            }
            .to_account_metas(None);

            self.account_metas.append(&mut pool_accounts);

            // Add an ATA where the pool transfer will come from
            let ata = get_associated_token_address(&self.margin_account, src_token);
            self.account_metas.push(AccountMeta::new(ata, false));
            self.spl_token_accounts.insert(*src_token);

            // Add the pool destination account
            let (pool_account, _) =
                owned_position_token_account(&self.margin_account, &pool.deposit_note_mint);
            self.account_metas
                .push(AccountMeta::new(pool_account, false));
            self.pool_note_mints.insert(pool.deposit_note_mint);
        }

        // Build accounts
        let mut accounts = pool.to_account_meta(src_token)?;

        self.account_metas.append(&mut accounts);

        // Update the route information and persist builder state
        let mut route = self
            .route_details
            .swap_routes
            .get_mut(self.next_route_index)
            .context("Unable to get route detail")?;
        if self.expects_multi_route {
            // This is the second leg of the multi-route
            route.route_b = pool.route_type();
            self.expects_multi_route = false;
            self.next_route_index += 1;
        } else {
            route.route_a = pool.route_type();
            route.split = swap_split;
            if swap_split > 0 {
                self.expects_multi_route = true;
            } else {
                self.next_route_index += 1;
            }
        }
        // Update the current tokens in the swap
        self.current_route_tokens = Some((*src_token, dst_token));

        Ok(())
    }

    /// Validate and finalize this swap
    pub fn finalize(&mut self) -> Result<()> {
        // Start with simple condiitions for data that should be present
        if self.next_route_index == 0 {
            bail!("No swap routes seem to be added")
        }
        if self.expects_multi_route {
            bail!("Swap incomplete, expected a second part of a swap to be executed as a split")
        }
        match &self.current_route_tokens {
            None => {
                bail!("There should be current route tokens populated in the swap")
            }
            Some((_, b)) => {
                if &self.dst_token != b {
                    bail!("Swap does not terminate in the provided destination token")
                }
            }
        }
        // Safe to finalize
        self.is_finalized = true;
        Ok(())
    }

    /// Get the instruction of the swap, which the caller should wrap with an invoke action
    pub fn get_instruction(&self) -> Result<Instruction> {
        // Check if finalized
        if !self.is_finalized {
            bail!("Can only get instruction when the builder is finalized")
        }
        Ok(Instruction {
            program_id: jet_margin_swap::id(),
            accounts: self.account_metas.clone(),
            data: self.route_details.data(),
        })
    }

    /// Get the pool note mints that are used in the instruction
    pub fn get_pool_note_mints(&self) -> &HashSet<Pubkey> {
        &self.pool_note_mints
    }

    /// Get SPL token accounts used in the transfer
    pub fn get_spl_token_mints(&self) -> &HashSet<Pubkey> {
        &self.spl_token_accounts
    }

    /// Verify that the swap can be added
    fn verify_addition(
        &self,
        src_token: &Pubkey,
        dst_token: &Pubkey,
        swap_split: u8,
    ) -> Result<()> {
        // If we are on the last index, we can only get a split
        if self.is_finalized {
            bail!("Cannot add route to a finalized swap");
        }
        if self.next_route_index > 2 {
            bail!("Cannot add more routes")
        }
        if self.expects_multi_route && swap_split > 0 {
            bail!("The next route is expected to be a second leg of a multi swap, do not specify percentage split");
        }
        // Check that the source token agrees with the expected next token
        if let Some((a, b)) = &self.current_route_tokens {
            // If on a multi-hop, the source and desitnation must agree, otherwise source = destination
            if self.expects_multi_route && (a != src_token || b != dst_token) {
                bail!("Source and destination tokens must be the same in a split-route swap")
            }
            if !self.expects_multi_route && b != src_token {
                // TODO: can word this error better
                bail!("The source token must be the same as the expected destination")
            }
        }
        // TODO: any other validations?

        Ok(())
    }
}

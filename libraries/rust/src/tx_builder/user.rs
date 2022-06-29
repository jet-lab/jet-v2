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

#![allow(unused)]

use std::sync::Arc;

use jet_metadata::{PositionTokenMetadata, TokenMetadata};

use anyhow::{bail, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{compute_budget::ComputeBudgetInstruction, instruction::Instruction};

use anchor_lang::AccountDeserialize;

use jet_margin::{MarginAccount, PositionKind};
use jet_margin_pool::Amount;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::ix_builder::*;

pub struct MarginTxBuilder {
    rpc: Arc<dyn SolanaRpcClient>,
    ix: MarginIxBuilder,
    signer: Option<Keypair>,
    is_liquidator: bool,
}

impl MarginTxBuilder {
    pub fn new(
        rpc: Arc<dyn SolanaRpcClient>,
        signer: Option<Keypair>,
        owner: Pubkey,
        seed: u16,
        is_liquidator: bool,
    ) -> MarginTxBuilder {
        let ix = MarginIxBuilder::new_with_payer(owner, seed, rpc.payer().pubkey());

        Self {
            rpc,
            ix,
            signer,
            is_liquidator,
        }
    }

    async fn create_transaction(&self, instructions: &[Instruction]) -> Result<Transaction> {
        let signers = self.signer.as_ref().map(|s| vec![s]).unwrap_or_default();

        self.rpc.create_transaction(&signers, instructions).await
    }

    async fn create_unsigned_transaction(
        &self,
        instructions: &[Instruction],
    ) -> Result<Transaction> {
        self.rpc.create_transaction(&[], instructions).await
    }

    pub fn signer(&self) -> Pubkey {
        self.signer.as_ref().unwrap().pubkey()
    }

    pub fn owner(&self) -> &Pubkey {
        &self.ix.owner
    }

    /// The address of the margin account
    pub fn address(&self) -> &Pubkey {
        &self.ix.address
    }

    /// Transaction to create a new margin account for the user
    pub async fn create_account(&self) -> Result<Transaction> {
        self.create_transaction(&[self.ix.create_account()]).await
    }

    /// Transaction to close the user's margin account
    pub async fn close_account(&self) -> Result<Transaction> {
        self.create_transaction(&[self.ix.close_account()]).await
    }

    /// Transaction to close the user's margin position accounts for a token mint.
    ///
    /// Both the deposit and loan position should be empty.
    /// Use [Self::close_empty_positions] to close all empty positions.
    pub async fn close_token_positions(&self, token_mint: &Pubkey) -> Result<Transaction> {
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let (deposit_account, _) = self.ix.get_token_account_address(&pool.deposit_note_mint);
        let (loan_account, _) = self.ix.get_token_account_address(&pool.loan_note_mint);
        let instructions = vec![
            self.ix
                .close_position(pool.deposit_note_mint, deposit_account),
            self.ix.close_position(pool.loan_note_mint, loan_account),
        ];
        self.create_transaction(&instructions).await
    }

    /// Transaction to close ther user's margin position account for a token mint and position king.
    ///
    /// The position should be empty.
    pub async fn close_token_position(
        &self,
        token_mint: &Pubkey,
        kind: PositionKind,
    ) -> Result<Transaction> {
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let ((account, _), mint) = match kind {
            PositionKind::NoValue | PositionKind::Deposit => (
                self.ix.get_token_account_address(&pool.deposit_note_mint),
                pool.deposit_note_mint,
            ),
            PositionKind::Claim => (
                self.ix.get_token_account_address(&pool.loan_note_mint),
                pool.loan_note_mint,
            ),
        };
        self.create_transaction(&[self.ix.close_position(mint, account)])
            .await
    }

    /// Transaction to close the user's empty position accounts.
    pub async fn close_empty_positions(&self) -> Result<Transaction> {
        let to_close = self
            .get_account_state()
            .await?
            .positions()
            .filter(|p| p.balance == 0)
            .map(|p| self.ix.close_position(p.token, p.address))
            .collect::<Vec<_>>();

        self.create_transaction(&to_close).await
    }

    /// Transaction to deposit tokens into a margin account
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens being deposited
    /// `source` - The token account that the deposit will be transfered from
    /// `amount` - The amount of tokens to deposit
    pub async fn deposit(
        &self,
        token_mint: &Pubkey,
        source: &Pubkey,
        amount: Amount,
    ) -> Result<Transaction> {
        let mut instructions = vec![];

        let pool = MarginPoolIxBuilder::new(*token_mint);
        let position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;

        instructions.push(pool.deposit(self.ix.owner, *source, position, amount));

        instructions.push(self.ix.update_position_balance(position));

        self.create_transaction(&instructions).await
    }

    /// Transaction to borrow tokens in a margin account
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens to borrow
    /// `amount` - The amount of tokens to borrow
    pub async fn borrow(&self, token_mint: &Pubkey, amount: Amount) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let token_metadata = self.get_token_metadata(token_mint).await?;

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let loan_position = self
            .get_or_create_position(&mut instructions, &pool.loan_note_mint)
            .await?;

        let inner_refresh_loan_ix =
            pool.margin_refresh_position(self.ix.address, token_metadata.pyth_price);
        instructions.push(self.adapter_invoke_ix(inner_refresh_loan_ix));

        let inner_borrow_ix =
            pool.margin_borrow(self.ix.address, deposit_position, loan_position, amount);

        instructions.push(self.adapter_invoke_ix(inner_borrow_ix));
        self.create_transaction(&instructions).await
    }

    /// Transaction to repay a loan of tokens in a margin account from the account's deposits
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens that were borrowed
    /// `amount` - The amount of tokens to repay
    pub async fn margin_repay(&self, token_mint: &Pubkey, amount: Amount) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let loan_position = self
            .get_or_create_position(&mut instructions, &pool.loan_note_mint)
            .await?;

        let inner_repay_ix =
            pool.margin_repay(self.ix.address, deposit_position, loan_position, amount);

        instructions.push(self.adapter_invoke_ix(inner_repay_ix));
        self.create_transaction(&instructions).await
    }

    /// Transaction to repay a loan of tokens in a margin account from a token account
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens that were borrowed
    /// `source` - The token account the repayment will be made from
    /// `amount` - The amount of tokens to repay
    pub async fn repay(
        &self,
        token_mint: &Pubkey,
        source: &Pubkey,
        amount: Amount,
    ) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let loan_position = self
            .get_or_create_position(&mut instructions, &pool.loan_note_mint)
            .await?;

        let inner_repay_ix = pool.repay(
            self.ix.address,
            self.ix.owner,
            *source,
            loan_position,
            amount,
        );

        instructions.push(self.adapter_invoke_ix(inner_repay_ix));
        self.create_transaction(&instructions).await
    }

    /// Transaction to withdraw tokens deposited into a margin account
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens to be withdrawn
    /// `amount` - The amount of tokens to withdraw
    pub async fn withdraw(
        &self,
        token_mint: &Pubkey,
        destination: &Pubkey,
        amount: Amount,
    ) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;

        let inner_withdraw_ix =
            pool.withdraw(self.ix.address, deposit_position, *destination, amount);

        instructions.push(self.adapter_invoke_ix(inner_withdraw_ix));
        self.create_transaction(&instructions).await
    }

    /// Transaction to swap one token for another
    ///
    /// # Notes
    ///
    /// - `transit_source_account` and `transit_destination_account` should be
    ///   created in a separate transaction to avoid packet size limits.
    #[allow(clippy::too_many_arguments)]
    pub async fn swap(
        &self,
        source_token_mint: &Pubkey,
        destination_token_mint: &Pubkey,
        transit_source_account: &Pubkey,
        transit_destination_account: &Pubkey,
        swap_pool: &Pubkey,
        pool_mint: &Pubkey,
        fee_account: &Pubkey,
        source_token_account: &Pubkey,
        destination_token_account: &Pubkey,
        swap_program: &Pubkey,
        amount_in: Amount,
        minimum_amount_out: Amount,
    ) -> Result<Transaction> {
        let mut instructions = vec![ComputeBudgetInstruction::request_units(300_000, 0)];
        let source_pool = MarginPoolIxBuilder::new(*source_token_mint);
        let destination_pool = MarginPoolIxBuilder::new(*destination_token_mint);

        let source_position = self
            .get_or_create_position(&mut instructions, &source_pool.deposit_note_mint)
            .await?;
        let destination_position = self
            .get_or_create_position(&mut instructions, &destination_pool.deposit_note_mint)
            .await?;

        let (swap_authority, _) = Pubkey::find_program_address(&[swap_pool.as_ref()], swap_program);
        let swap_pool = MarginSwapIxBuilder::new(
            *source_token_mint,
            *destination_token_mint,
            *swap_pool,
            swap_authority,
            *pool_mint,
            *fee_account,
        );

        let inner_swap_ix = swap_pool.swap(
            *self.address(),
            *transit_source_account,
            *transit_destination_account,
            source_position,
            destination_position,
            *source_token_account,
            *destination_token_account,
            *swap_program,
            &source_pool,
            &destination_pool,
            amount_in.value,
            minimum_amount_out.value,
        );

        instructions.push(self.adapter_invoke_ix(inner_swap_ix));

        self.create_transaction(&instructions).await
    }

    /// Transaction to begin liquidating user account.
    /// If `refresh_position` is provided, all the margin pools will be refreshed first.
    pub async fn liquidate_begin(&self, refresh_positions: bool) -> Result<Transaction> {
        assert!(self.is_liquidator);

        // Get the margin account and refresh positions
        let mut instructions = vec![];
        if refresh_positions {
            self.create_pool_instructions(&mut instructions).await?;
        }

        // Add liquidation instruction
        instructions.push(
            self.ix
                .liquidate_begin(self.signer.as_ref().unwrap().pubkey()),
        );
        self.create_transaction(&instructions).await
    }

    /// Transaction to end liquidating user account
    pub async fn liquidate_end(&self, original_liquidator: Option<Pubkey>) -> Result<Transaction> {
        let self_key = self
            .signer
            .as_ref()
            .map(|s| s.pubkey())
            .unwrap_or(*self.owner());
        self.create_transaction(&[self.ix.liquidate_end(self_key, original_liquidator)])
            .await
    }

    /// Verify that the margin account is healthy
    pub async fn verify_healthy(&self) -> Result<Transaction> {
        self.create_unsigned_transaction(&[self.ix.verify_healthy()])
            .await
    }

    /// Refresh a user's position in a margin pool
    pub async fn refresh_pool_position(&self, token_mint: &Pubkey) -> Result<Transaction> {
        let metadata = self.get_token_metadata(token_mint).await?;
        let ix_builder = MarginPoolIxBuilder::new(*token_mint);
        let ix = self.ix.adapter_invoke(
            ix_builder.margin_refresh_position(self.ix.address, metadata.pyth_price),
        );

        self.create_transaction(&[ix]).await
    }

    /// Refresh all of a user's positions based in the margin pool
    pub async fn refresh_all_pool_positions(&self) -> Result<Vec<Transaction>> {
        let mut instructions = vec![];
        self.create_pool_instructions(&mut instructions).await?;

        self.get_chunk_transactions(12, instructions).await
    }

    /// Refresh the metadata for a position
    pub async fn refresh_position_metadata(
        &self,
        position_token_mint: &Pubkey,
    ) -> Result<Transaction> {
        self.create_transaction(&[self.ix.refresh_position_metadata(position_token_mint)])
            .await
    }

    /// Refresh metadata for all positions in the user account
    pub async fn refresh_all_position_metadata(&self) -> Result<Vec<Transaction>> {
        let mut instructions = self
            .get_account_state()
            .await?
            .positions()
            .map(|position| self.ix.refresh_position_metadata(&position.token))
            .collect::<Vec<_>>();

        self.get_chunk_transactions(12, instructions).await
    }

    async fn get_chunk_transactions(
        &self,
        chunk_size: usize,
        instructions: Vec<Instruction>,
    ) -> Result<Vec<Transaction>> {
        futures::future::join_all(
            instructions
                .chunks(chunk_size)
                .map(|c| self.create_unsigned_transaction(c)),
        )
        .await
        .into_iter()
        .collect()
    }

    /// Append instructions to refresh pool positions to instructions
    async fn create_pool_instructions(&self, instructions: &mut Vec<Instruction>) -> Result<()> {
        let state = self.get_account_state().await?;
        let count = state.positions().count();

        for position in state.positions() {
            let p_metadata = self.get_position_metadata(&position.token).await?;
            let t_metadata = self
                .get_token_metadata(&p_metadata.underlying_token_mint)
                .await?;
            let ix_builder = MarginPoolIxBuilder::new(p_metadata.underlying_token_mint);
            let ix = self.ix.accounting_invoke(
                ix_builder.margin_refresh_position(self.ix.address, t_metadata.pyth_price),
            );

            instructions.push(ix);
        }

        Ok(())
    }

    async fn get_token_metadata(&self, token_mint: &Pubkey) -> Result<TokenMetadata> {
        let (md_address, _) =
            Pubkey::find_program_address(&[token_mint.as_ref()], &jet_metadata::ID);
        let account_data = self.rpc.get_account(&md_address).await?;

        match account_data {
            None => bail!("no metadata {} found for token {}", md_address, token_mint),
            Some(account) => Ok(TokenMetadata::try_deserialize(&mut &account.data[..])?),
        }
    }

    async fn get_position_metadata(
        &self,
        position_token_mint: &Pubkey,
    ) -> Result<PositionTokenMetadata> {
        let (md_address, _) =
            Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

        let account_data = self.rpc.get_account(&md_address).await?;

        match account_data {
            None => bail!(
                "no metadata {} found for position token {}",
                md_address,
                position_token_mint
            ),
            Some(account) => Ok(PositionTokenMetadata::try_deserialize(
                &mut &account.data[..],
            )?),
        }
    }

    async fn get_or_create_position(
        &self,
        instructions: &mut Vec<Instruction>,
        token_mint: &Pubkey,
    ) -> Result<Pubkey> {
        let state = self.get_account_state().await?;
        let (address, ix_register) = self.ix.register_position(*token_mint);

        if !state.positions().any(|p| p.token == *token_mint) {
            instructions.push(ix_register);
        }

        Ok(address)
    }

    async fn get_account_state(&self) -> Result<Box<MarginAccount>> {
        let account_data = self.rpc.get_account(&self.ix.address).await?;

        match account_data {
            None => bail!(
                "no account state found for account {} belonging to {}",
                self.ix.owner,
                self.ix.address
            ),
            Some(account) => Ok(Box::new(MarginAccount::try_deserialize(
                &mut &account.data[..],
            )?)),
        }
    }

    fn adapter_invoke_ix(&self, inner: Instruction) -> Instruction {
        match self.is_liquidator {
            true => self
                .ix
                .liquidator_invoke(inner, &self.signer.as_ref().unwrap().pubkey()),
            false => self.ix.adapter_invoke(inner),
        }
    }
}

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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use jet_margin_pool::program::JetMarginPool;
use jet_metadata::{PositionTokenMetadata, TokenMetadata};

use anyhow::{bail, Result};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use anchor_lang::{AccountDeserialize, Id};

use jet_margin::{MarginAccount, PositionKind};
use jet_margin_pool::{Amount, TokenChange};
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::{
    ix_builder::*,
    solana::{
        keypair::clone,
        transaction::{SendTransactionBuilder, TransactionBuilder},
    },
};

/// [Transaction] builder for a margin account, which supports invoking adapter
/// actions signed as the margin account.
/// Actions are invoked through `adapter_invoke_ix` depending on their context.
///
/// Both margin accounts and liquidators can use this builder, and it will invoke
/// the correct `adapter_invoke_ix`.
pub struct MarginTxBuilder {
    rpc: Arc<dyn SolanaRpcClient>,
    ix: MarginIxBuilder,
    signer: Option<Keypair>,
    is_liquidator: bool,
}

impl Clone for MarginTxBuilder {
    fn clone(&self) -> Self {
        Self {
            rpc: self.rpc.clone(),
            ix: self.ix.clone(),
            signer: self
                .signer
                .as_ref()
                .map(|kp| Keypair::from_bytes(&kp.to_bytes()).unwrap()),
            is_liquidator: self.is_liquidator,
        }
    }
}

impl MarginTxBuilder {
    /// Create a [MarginTxBuilder] for an ordinary user. Liquidators should use
    /// `Self::new_liquidator`.
    pub fn new(
        rpc: Arc<dyn SolanaRpcClient>,
        signer: Option<Keypair>,
        owner: Pubkey,
        seed: u16,
    ) -> MarginTxBuilder {
        let ix = MarginIxBuilder::new_with_payer(owner, seed, rpc.payer().pubkey(), None);

        Self {
            rpc,
            ix,
            signer,
            is_liquidator: false,
        }
    }

    /// Createa  new [MarginTxBuilder] for a liquidator. Sets the liquidator
    /// as the authority when interacting with the margin program.
    ///
    /// A liquidator is almost always the payer of the transaction,
    /// their pubkey would be the same as `rpc.payer()`, however we explicitly
    /// supply it to support cases where the liquidator is not the fee payer.
    pub fn new_liquidator(
        rpc: Arc<dyn SolanaRpcClient>,
        signer: Option<Keypair>,
        owner: Pubkey,
        seed: u16,
        liquidator: Pubkey,
    ) -> MarginTxBuilder {
        let ix =
            MarginIxBuilder::new_with_payer(owner, seed, rpc.payer().pubkey(), Some(liquidator));

        Self {
            rpc,
            ix,
            signer,
            is_liquidator: true,
        }
    }

    async fn create_transaction(&self, instructions: &[Instruction]) -> Result<Transaction> {
        let signers = self.signer.as_ref().map(|s| vec![s]).unwrap_or_default();

        self.rpc.create_transaction(&signers, instructions).await
    }

    fn create_transaction_builder(
        &self,
        instructions: &[Instruction],
    ) -> Result<TransactionBuilder> {
        let signers = self
            .signer
            .as_ref()
            .map(|s| vec![clone(s)])
            .unwrap_or_default();

        Ok(TransactionBuilder {
            signers,
            instructions: instructions.to_vec(),
        })
    }

    async fn create_unsigned_transaction(
        &self,
        instructions: &[Instruction],
    ) -> Result<Transaction> {
        self.rpc.create_transaction(&[], instructions).await
    }

    /// The address of the transaction signer
    pub fn signer(&self) -> Pubkey {
        self.signer.as_ref().unwrap().pubkey()
    }

    /// The owner of the margin account
    pub fn owner(&self) -> &Pubkey {
        &self.ix.owner
    }

    /// The address of the margin account
    pub fn address(&self) -> &Pubkey {
        &self.ix.address
    }

    /// The seed of the margin account
    pub fn seed(&self) -> u16 {
        self.ix.seed
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
        let instructions = vec![
            self.ix
                .close_position(pool.deposit_note_mint, deposit_account),
            self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer)),
        ];
        self.create_transaction(&instructions).await
    }

    /// Transaction to close ther user's margin position account for a token mint and position king.
    ///
    /// The position should be empty.
    pub async fn close_pool_position(
        &self,
        token_mint: &Pubkey,
        kind: PositionKind,
    ) -> Result<Transaction> {
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let ix = match kind {
            PositionKind::NoValue | PositionKind::Deposit => self.ix.close_position(
                pool.deposit_note_mint,
                self.ix.get_token_account_address(&pool.deposit_note_mint).0,
            ),
            PositionKind::Claim => {
                self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer))
            }
        };

        self.create_transaction(&[ix]).await
    }

    /// Transaction to close the user's empty position accounts.
    pub async fn close_empty_positions(
        &self,
        loan_to_token: &HashMap<Pubkey, Pubkey>,
    ) -> Result<Transaction> {
        let to_close = self
            .get_account_state()
            .await?
            .positions()
            .filter(|p| p.balance == 0)
            .map(|p| {
                if p.adapter == JetMarginPool::id() && p.kind() == PositionKind::Claim {
                    let pool = MarginPoolIxBuilder::new(*loan_to_token.get(&p.token).unwrap());
                    self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer))
                } else {
                    self.ix.close_position(p.token, p.address)
                }
            })
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
        change: TokenChange,
    ) -> Result<Transaction> {
        let mut instructions = vec![];

        let pool = MarginPoolIxBuilder::new(*token_mint);
        let position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;

        instructions.push(pool.deposit(self.ix.owner, *source, position, change));

        instructions.push(self.ix.update_position_balance(position));

        self.create_transaction(&instructions).await
    }

    /// Transaction to borrow tokens in a margin account
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens to borrow
    /// `amount` - The amount of tokens to borrow
    pub async fn borrow(&self, token_mint: &Pubkey, change: TokenChange) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let token_metadata = self.get_token_metadata(token_mint).await?;

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let loan_position = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_refresh_loan_ix =
            pool.margin_refresh_position(self.ix.address, token_metadata.pyth_price);
        instructions.push(self.adapter_invoke_ix(inner_refresh_loan_ix));

        let inner_borrow_ix =
            pool.margin_borrow(self.ix.address, deposit_position, loan_position, change);

        instructions.push(self.adapter_invoke_ix(inner_borrow_ix));
        self.create_transaction(&instructions).await
    }

    /// Transaction to repay a loan of tokens in a margin account from the account's deposits
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens that were borrowed
    /// `amount` - The amount of tokens to repay
    pub async fn margin_repay(
        &self,
        token_mint: &Pubkey,
        change: TokenChange,
    ) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let loan_position = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_repay_ix =
            pool.margin_repay(self.ix.address, deposit_position, loan_position, change);

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
        change: TokenChange,
    ) -> Result<Transaction> {
        let mut instructions = vec![];

        let pool = MarginPoolIxBuilder::new(*token_mint);
        let loan_position = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_repay_ix = pool.repay(self.ix.owner, *source, loan_position, change);

        instructions.push(inner_repay_ix);
        instructions.push(self.ix.update_position_balance(loan_position));

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
        change: TokenChange,
    ) -> Result<Transaction> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;

        let inner_withdraw_ix =
            pool.withdraw(self.ix.address, deposit_position, *destination, change);

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
        let mut instructions = vec![];
        let source_pool = MarginPoolIxBuilder::new(*source_token_mint);
        let destination_pool = MarginPoolIxBuilder::new(*destination_token_mint);

        let source_position = self
            .get_or_create_position(&mut instructions, &source_pool.deposit_note_mint)
            .await?;
        let destination_position = self
            .get_or_create_position(&mut instructions, &destination_pool.deposit_note_mint)
            .await?;

        let destination_metadata = self.get_token_metadata(destination_token_mint).await?;

        // Only refreshing the destination due to transaction size.
        // The most common scenario would be that a new margin position is created
        // for the destination of the swap. If its position price is not set before
        // the swap, a liquidator would be accused of extracting too much value
        // as the destination becomes immediately stale after creation.
        instructions.push(
            self.ix.accounting_invoke(
                destination_pool
                    .margin_refresh_position(*self.address(), destination_metadata.pyth_price),
            ),
        );

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
            amount_in.value(),
            minimum_amount_out.value(),
        );

        instructions.push(self.adapter_invoke_ix(inner_swap_ix));

        self.create_transaction(&instructions).await
    }

    /// Transaction to begin liquidating user account.
    /// If `refresh_position` is provided, all the margin pools will be refreshed first.
    pub async fn liquidate_begin(&self, refresh_positions: bool) -> Result<Transaction> {
        let builder = self.liquidate_begin_builder(refresh_positions).await?;
        self.rpc.compile(builder).await
    }

    /// Transaction to begin liquidating user account.
    /// If `refresh_position` is provided, all the margin pools will be refreshed first.
    pub async fn liquidate_begin_builder(
        &self,
        refresh_positions: bool,
    ) -> Result<TransactionBuilder> {
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

        self.create_transaction_builder(&instructions)
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
        let instructions = self
            .get_account_state()
            .await?
            .positions()
            .map(|position| self.ix.refresh_position_metadata(&position.token))
            .collect::<Vec<_>>();

        self.get_chunk_transactions(12, instructions).await
    }

    /// Get the latest [MarginAccount] state
    pub async fn get_account_state(&self) -> Result<Box<MarginAccount>> {
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
        let mut seen_pools = HashSet::new();

        for position in state.positions() {
            let p_metadata = self.get_position_metadata(&position.token).await?;
            if seen_pools.contains(&p_metadata.underlying_token_mint) {
                continue;
            }
            let t_metadata = self
                .get_token_metadata(&p_metadata.underlying_token_mint)
                .await?;
            let ix_builder = MarginPoolIxBuilder::new(p_metadata.underlying_token_mint);
            let ix = self.ix.accounting_invoke(
                ix_builder.margin_refresh_position(self.ix.address, t_metadata.pyth_price),
            );

            instructions.push(ix);
            seen_pools.insert(p_metadata.underlying_token_mint);
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

    async fn get_or_create_pool_loan_position(
        &self,
        instructions: &mut Vec<Instruction>,
        pool: &MarginPoolIxBuilder,
    ) -> Result<Pubkey> {
        let state = self.get_account_state().await?;
        let search_result = state.positions().find(|p| p.token == pool.loan_note_mint);

        Ok(if let Some(position) = search_result {
            position.address
        } else {
            let (loan_note_token_account, pools_ix) =
                pool.register_loan(self.ix.address, self.ix.payer);
            let wrapped_ix = self.adapter_invoke_ix(pools_ix);
            instructions.push(wrapped_ix);

            loan_note_token_account
        })
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

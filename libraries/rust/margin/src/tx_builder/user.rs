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

use std::collections::HashMap;
use std::sync::Arc;

use anchor_spl::associated_token::get_associated_token_address;
use async_trait::async_trait;
use jet_margin_pool::program::JetMarginPool;
use jet_metadata::{PositionTokenMetadata, TokenMetadata};

use anyhow::{bail, Result};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use anchor_lang::{AccountDeserialize, Id};

use jet_margin::{MarginAccount, TokenConfig, TokenKind, TokenOracle};
use jet_margin_pool::TokenChange;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::cat;
use crate::lookup_tables::LookupTable;
use crate::margin_integrator::PositionRefresher;
use crate::solana::transaction::WithSigner;
use crate::util::data::Join;
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
    /// builds the instructions for margin without any rpc interaction or
    /// knowledge of other programs
    pub ix: MarginIxBuilder,
    config_ix: MarginConfigIxBuilder,
    signer: Option<Keypair>,
    is_liquidator: bool,
}

impl Clone for MarginTxBuilder {
    fn clone(&self) -> Self {
        Self {
            rpc: self.rpc.clone(),
            ix: self.ix.clone(),
            config_ix: self.config_ix.clone(),
            signer: self
                .signer
                .as_ref()
                .map(|kp| Keypair::from_bytes(&kp.to_bytes()).unwrap()),
            is_liquidator: self.is_liquidator,
        }
    }
}

#[async_trait]
impl PositionRefresher for MarginTxBuilder {
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>> {
        Ok(cat![
            self.refresh_all_pool_positions().await?,
            self.refresh_deposit_positions().await?,
        ])
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
        airspace: Pubkey,
    ) -> MarginTxBuilder {
        let mut ix = MarginIxBuilder::new(airspace, owner, seed).with_payer(rpc.payer().pubkey());
        if let Some(signer) = signer.as_ref() {
            ix = ix.with_authority(signer.pubkey());
        }
        let config_ix = MarginConfigIxBuilder::new(airspace, rpc.payer().pubkey(), None);

        Self {
            rpc,
            ix,
            config_ix,
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
        airspace: Pubkey,
        owner: Pubkey,
        seed: u16,
    ) -> MarginTxBuilder {
        let mut ix = MarginIxBuilder::new(airspace, owner, seed).with_payer(rpc.payer().pubkey());
        if let Some(signer) = signer.as_ref() {
            ix = ix.with_authority(signer.pubkey());
        }
        let config_ix = MarginConfigIxBuilder::new(Pubkey::default(), rpc.payer().pubkey(), None);

        Self {
            rpc,
            ix,
            config_ix,
            signer,
            is_liquidator: true,
        }
    }

    /// Creates a variant of the builder that has a signer other than the payer.
    pub fn with_signer(mut self, signer: Keypair) -> Self {
        self.ix = self.ix.with_authority(signer.pubkey());
        self.signer = Some(signer);

        self
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

    /// The address of the transaction signer
    fn signers(&self) -> Vec<Keypair> {
        match &self.signer {
            Some(s) => vec![clone(s)],
            None => vec![],
        }
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

    /// The address of the associated airspace
    pub fn airspace(&self) -> Pubkey {
        self.ix.airspace
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
        let deposit_account = self.ix.get_token_account_address(&pool.deposit_note_mint);
        let instructions = vec![
            self.ix
                .close_position(pool.deposit_note_mint, deposit_account),
            self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer())),
        ];
        self.create_transaction(&instructions).await
    }

    /// Transaction to close ther user's margin position account for a token mint and position king.
    ///
    /// The position should be empty.
    pub async fn close_pool_position(
        &self,
        token_mint: &Pubkey,
        kind: TokenKind,
    ) -> Result<Transaction> {
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let ix = match kind {
            TokenKind::Collateral => self.ix.close_position(
                pool.deposit_note_mint,
                self.ix.get_token_account_address(&pool.deposit_note_mint),
            ),
            TokenKind::Claim => {
                self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer()))
            }
            TokenKind::AdapterCollateral => panic!("pools do not issue AdapterCollateral"),
        };

        self.create_transaction(&[ix]).await
    }

    /// Transaction to close the user's empty position accounts.
    pub async fn close_empty_positions(
        &self,
        loan_to_token: &HashMap<Pubkey, Pubkey>,
    ) -> Result<TransactionBuilder> {
        let to_close = self
            .get_account_state()
            .await?
            .positions()
            .filter(|p| p.balance == 0)
            .map(|p| {
                if p.adapter == JetMarginPool::id() && p.kind() == TokenKind::Claim {
                    let pool = MarginPoolIxBuilder::new(*loan_to_token.get(&p.token).unwrap());
                    self.adapter_invoke_ix(pool.close_loan(*self.address(), self.ix.payer()))
                } else {
                    self.ix.close_position(p.token, p.address)
                }
            })
            .collect::<Vec<_>>();

        self.create_transaction_builder(&to_close)
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
    pub async fn borrow(
        &self,
        token_mint: &Pubkey,
        change: TokenChange,
    ) -> Result<TransactionBuilder> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let token_metadata = self.get_token_metadata(token_mint).await?;

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let _ = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_refresh_loan_ix =
            pool.margin_refresh_position(self.ix.address, token_metadata.pyth_price);
        instructions.push(self.ix.accounting_invoke(inner_refresh_loan_ix));

        let inner_borrow_ix = pool.margin_borrow(self.ix.address, deposit_position, change);

        instructions.push(self.adapter_invoke_ix(inner_borrow_ix));
        self.create_transaction_builder(&instructions)
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
    ) -> Result<TransactionBuilder> {
        let mut instructions = vec![];
        let pool = MarginPoolIxBuilder::new(*token_mint);

        let deposit_position = self
            .get_or_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let _ = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_repay_ix = pool.margin_repay(self.ix.address, deposit_position, change);

        instructions.push(self.adapter_invoke_ix(inner_repay_ix));
        self.create_transaction_builder(&instructions)
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
        swap_pool: &Pubkey,
        pool_mint: &Pubkey,
        fee_account: &Pubkey,
        source_token_account: &Pubkey,
        destination_token_account: &Pubkey,
        swap_program: &Pubkey,
        // for levswap
        change: TokenChange,
        minimum_amount_out: u64,
    ) -> Result<TransactionBuilder> {
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

        let swap_info = SplSwap {
            program: *swap_program,
            address: *swap_pool,
            pool_mint: *pool_mint,
            token_a: *source_token_mint,
            token_b: *destination_token_mint,
            token_a_vault: *source_token_account,
            token_b_vault: *destination_token_account,
            fee_account: *fee_account,
        };
        let inner_swap_ix = pool_spl_swap(
            &swap_info,
            &self.airspace(),
            &self.ix.address,
            source_token_mint,
            destination_token_mint,
            change.kind,
            change.tokens,
            minimum_amount_out,
        );

        instructions.push(self.adapter_invoke_ix(inner_swap_ix));

        instructions.push(self.ix.update_position_balance(source_position));
        instructions.push(self.ix.update_position_balance(destination_position));

        self.create_transaction_builder(&instructions)
    }

    /// Transaction to swap tokens in a chain of up to 3 legs.
    ///
    /// The function accepts the instruction route builder which is expected to be finalized.
    pub async fn route_swap_with_lookup(
        &self,
        builder: &MarginSwapRouteIxBuilder,
        account_lookup_tables: &[Pubkey],
        signer: &Keypair,
    ) -> Result<VersionedTransaction> {
        // We can't get the instruction if not finalized, get it to check.
        let inner_swap_ix = builder.get_instruction()?;

        let mut instructions = self.setup_swap(builder).await?;

        instructions.push(self.adapter_invoke_ix(inner_swap_ix));

        let tx = LookupTable::use_lookup_tables(
            &self.rpc,
            account_lookup_tables,
            &instructions,
            &[signer],
        )
        .await?;
        Ok(tx)
    }

    /// Transaction to swap tokens in a chain of up to 3 legs.
    ///
    /// The function accepts the instruction route builder which is expected to be finalized.
    pub async fn route_swap(
        &self,
        builder: &MarginSwapRouteIxBuilder,
    ) -> Result<Vec<TransactionBuilder>> {
        // We can't get the instruction if not finalized, get it to check.
        let inner_swap_ix = builder.get_instruction()?;

        let setup_instructions = self.setup_swap(builder).await?;
        let mut transactions = vec![];
        if !setup_instructions.is_empty() {
            transactions.push(self.create_transaction_builder(&setup_instructions)?);
        }
        transactions.push(self.create_transaction_builder(&[
            ComputeBudgetInstruction::set_compute_unit_limit(800000),
            self.adapter_invoke_ix(inner_swap_ix),
        ])?);

        Ok(transactions)
    }

    async fn setup_swap(&self, builder: &MarginSwapRouteIxBuilder) -> Result<Vec<Instruction>> {
        let mut setup_instructions = vec![];
        for deposit_note_mint in builder.get_pool_note_mints() {
            self.get_or_create_position(&mut setup_instructions, deposit_note_mint)
                .await?;
        }
        for token_mint in builder.get_spl_token_mints() {
            // Check if an ATA exists before creating it
            // TODO: if swapping using margin tokens, we could register positions
            let ata = get_associated_token_address(self.address(), token_mint);
            if self.rpc.get_account(&ata).await?.is_none() {
                let ix = spl_associated_token_account::instruction::create_associated_token_account(
                    &self.signer(),
                    self.address(),
                    token_mint,
                    &spl_token::id(),
                );
                setup_instructions.push(ix);
            }
        }
        Ok(setup_instructions)
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
        let mut txs = if refresh_positions {
            cat![
                self.refresh_all_pool_positions().await?,
                self.refresh_deposit_positions().await?,
            ]
            .ijoin()
        } else {
            TransactionBuilder::default()
        };

        // Add liquidation instruction
        txs.instructions.push(self.ix.liquidate_begin());
        txs.signers.push(clone(self.signer.as_ref().unwrap()));

        Ok(txs)
    }

    /// Transaction to end liquidating user account
    pub async fn liquidate_end(&self, original_liquidator: Option<Pubkey>) -> Result<Transaction> {
        self.create_transaction(&[self.ix.liquidate_end(original_liquidator)])
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

    /// Append instructions to refresh pool positions to instructions
    pub async fn refresh_all_pool_positions(&self) -> Result<Vec<TransactionBuilder>> {
        Ok(self
            .refresh_all_pool_positions_underlying_to_tx()
            .await?
            .into_values()
            .collect())
    }

    /// Refresh metadata for all positions in the user account
    pub async fn refresh_all_position_metadata(&self) -> Result<Vec<TransactionBuilder>> {
        let instructions = self
            .get_account_state()
            .await?
            .positions()
            .map(|position| {
                self.ix
                    .refresh_position_config(&position.token)
                    .with_signers(&self.signers())
            })
            .collect::<Vec<_>>();

        Ok(instructions)
    }

    /// Create a new token account that accepts deposits, registered as a position
    pub async fn create_deposit_position(&self, token_mint: &Pubkey) -> Result<Transaction> {
        self.create_transaction(&[
            spl_associated_token_account::instruction::create_associated_token_account(
                &self.signer(),
                self.address(),
                token_mint,
                &spl_token::id(),
            ),
            self.ix.create_deposit_position(*token_mint),
        ])
        .await
    }

    /// Close a previously created deposit account
    pub async fn close_deposit_position(&self, token_mint: &Pubkey) -> Result<Transaction> {
        let token_account = get_associated_token_address(self.address(), token_mint);
        let instruction = self.ix.close_position(*token_mint, token_account);
        self.create_transaction(&[instruction]).await
    }

    /// Transfer tokens into or out of a deposit account associated with the margin account
    pub async fn transfer_deposit(
        &self,
        token_mint: Pubkey,
        source_owner: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        amount: u64,
    ) -> Result<Transaction> {
        let state = self.get_account_state().await?;
        let mut instructions = vec![];

        if !state.positions().any(|p| p.token == token_mint) {
            instructions.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &self.signer(),
                    self.address(),
                    &token_mint,
                    &spl_token::id(),
                ),
            );
            instructions.push(self.ix.create_deposit_position(token_mint));
        }

        instructions.push(
            self.ix
                .transfer_deposit(source_owner, source, destination, amount),
        );

        self.create_transaction(&instructions).await
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

    /// Append instructions to refresh pool positions to instructions
    pub async fn refresh_all_pool_positions_underlying_to_tx(
        &self,
    ) -> Result<HashMap<Pubkey, TransactionBuilder>> {
        let state = self.get_account_state().await?;
        let mut txns = HashMap::new();

        for position in state.positions() {
            if position.adapter != jet_margin_pool::ID {
                continue;
            }
            let p_metadata = self.get_position_metadata(&position.token).await?;
            if txns.contains_key(&p_metadata.underlying_token_mint) {
                continue;
            }
            let t_metadata = self
                .get_token_metadata(&p_metadata.underlying_token_mint)
                .await?;
            let ix_builder = MarginPoolIxBuilder::new(p_metadata.underlying_token_mint);
            let ix = self.ix.accounting_invoke(
                ix_builder.margin_refresh_position(self.ix.address, t_metadata.pyth_price),
            );

            txns.insert(p_metadata.underlying_token_mint, ix.into());
        }

        Ok(txns)
    }

    /// Append instructions to refresh deposit positions
    pub async fn refresh_deposit_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let state = self.get_account_state().await?;
        let mut instructions = vec![];
        for position in state.positions() {
            let (cfg_addr, p_config) = match self.get_position_config(&position.token).await? {
                None => continue,
                Some(r) => r,
            };

            if position.token != p_config.underlying_mint {
                continue;
            }

            let token_oracle = match p_config.oracle().unwrap() {
                TokenOracle::Pyth { price, .. } => price,
            };

            let refresh = self.ix.refresh_deposit_position(&cfg_addr, &token_oracle);
            instructions.push(refresh.into());
        }

        Ok(instructions)
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

    async fn get_position_config(
        &self,
        token_mint: &Pubkey,
    ) -> Result<Option<(Pubkey, TokenConfig)>> {
        let cfg_address = self.config_ix.derive_token_config(token_mint);
        let account_data = self.rpc.get_account(&cfg_address).await?;

        match account_data {
            None => Ok(None),
            Some(account) => Ok(Some((
                cfg_address,
                TokenConfig::try_deserialize(&mut &account.data[..])?,
            ))),
        }
    }

    async fn get_or_create_position(
        &self,
        instructions: &mut Vec<Instruction>,
        token_mint: &Pubkey,
    ) -> Result<Pubkey> {
        let state = self.get_account_state().await?;
        let ix_register = self.ix.register_position(*token_mint);

        if !state.positions().any(|p| p.token == *token_mint) {
            instructions.push(ix_register);
        }

        Ok(derive_position_token_account(&self.ix.address, token_mint))
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
            let pools_ix = pool.register_loan(self.ix.address, self.ix.payer(), self.airspace());
            let wrapped_ix = self.adapter_invoke_ix(pools_ix);
            instructions.push(wrapped_ix);

            derive_loan_account(&self.ix.address, &pool.loan_note_mint)
        })
    }

    fn adapter_invoke_ix(&self, inner: Instruction) -> Instruction {
        match self.is_liquidator {
            true => self.ix.liquidator_invoke(inner),
            false => self.ix.adapter_invoke(inner),
        }
    }
}

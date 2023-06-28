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
use jet_instructions::openbook::{close_open_orders, create_open_orders};
use jet_margin_pool::program::JetMarginPool;

use anyhow::{Context, Result};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use anchor_lang::{AccountDeserialize, Id};

use jet_margin::{MarginAccount, TokenKind};
use jet_margin_pool::{MarginPool, TokenChange};
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::cat;
use crate::get_state::{get_margin_account, get_token_metadata};
use crate::lookup_tables::LookupTable;
use crate::margin_account_ext::MarginAccountExt;
use crate::refresh::deposit::refresh_deposit_positions;
use crate::refresh::pool::{
    refresh_all_pool_positions, refresh_all_pool_positions_underlying_to_tx,
};
use crate::refresh::position_refresher::{HasMarginAccountAddress, HasRpc, PositionRefresher};
use crate::solana::pubkey::OrAta;
use crate::solana::transaction::WithSigner;
use crate::util::data::Join;
use crate::{
    ix_builder::*,
    solana::{
        keypair::{clone, KeypairExt},
        transaction::{SendTransactionBuilder, TransactionBuilder},
    },
};

use super::invoke_pool::PoolTargetPosition;
use super::MarginInvokeContext;

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
impl PositionRefresher<MarginAccount> for MarginTxBuilder {
    async fn refresh_positions(
        &self,
        margin_account: &MarginAccount,
    ) -> Result<Vec<TransactionBuilder>> {
        Ok(cat![
            refresh_all_pool_positions(&self.rpc, margin_account).await?,
            refresh_deposit_positions(&self.rpc, margin_account).await?,
        ])
    }
}
impl HasRpc for MarginTxBuilder {
    fn rpc(&self) -> Arc<dyn SolanaRpcClient> {
        self.rpc.clone()
    }
}
impl HasMarginAccountAddress for MarginTxBuilder {
    fn margin_account_address(&self) -> Pubkey {
        self.ix.address
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
        liquidator: Keypair,
        airspace: Pubkey,
        owner: Pubkey,
        seed: u16,
    ) -> MarginTxBuilder {
        let ix = MarginIxBuilder::new(airspace, owner, seed)
            .with_payer(rpc.payer().pubkey())
            .with_authority(liquidator.pubkey());
        let config_ix = MarginConfigIxBuilder::new(airspace, rpc.payer().pubkey(), None);

        Self {
            rpc,
            ix,
            config_ix,
            signer: Some(liquidator),
            is_liquidator: true,
        }
    }

    /// returns None if there is no signer.
    pub fn invoke_ctx(&self) -> MarginInvokeContext {
        MarginInvokeContext {
            airspace: self.airspace(),
            margin_account: *self.address(),
            authority: MarginActionAuthority::AccountAuthority.resolve(&self.ix),
            is_liquidator: self.is_liquidator,
        }
    }

    /// whether the current builder is for a liquidator
    pub fn is_liquidator(&self) -> bool {
        self.is_liquidator
    }

    /// Creates a new Self for actions on the same margin account, but
    /// authorized by provided liquidator.
    pub fn liquidator(&self, liquidator: Keypair) -> Self {
        Self {
            rpc: self.rpc.clone(),
            ix: self.ix.clone().with_authority(liquidator.pubkey()),
            config_ix: self.config_ix.clone(),
            signer: Some(liquidator),
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

    fn create_transaction_builder(&self, instructions: &[Instruction]) -> TransactionBuilder {
        let signers = self
            .signer
            .as_ref()
            .map(|s| vec![s.clone()])
            .unwrap_or_default();

        TransactionBuilder {
            signers,
            instructions: instructions.to_vec(),
        }
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

    /// Transaction to create an address lookup registry account
    pub async fn init_lookup_registry(&self) -> Result<Transaction> {
        self.create_transaction(&[self.ix.init_lookup_registry()])
            .await
    }

    /// Transaction to create a lookup table account
    pub async fn create_lookup_table(&self) -> Result<(Transaction, Pubkey)> {
        let recent_slot = self
            .rpc
            .get_slot(Some(CommitmentConfig::finalized()))
            .await?;
        let (ix, lookup_table) = self.ix.create_lookup_table(recent_slot);
        let tx = self.create_transaction(&[ix]).await?;

        Ok((tx, lookup_table))
    }

    /// Transaction to append accounts to a lookup table account
    pub async fn append_to_lookup_table(
        &self,
        lookup_table: Pubkey,
        addresses: &[Pubkey],
    ) -> Result<Transaction> {
        self.create_transaction(&[self.ix.append_to_lookup_table(lookup_table, addresses)])
            .await
    }

    /// Transaction to close the user's margin position accounts for a token mint.
    ///
    /// Both the deposit and loan position should be empty.
    /// Use [Self::close_empty_positions] to close all empty positions.
    pub async fn close_pool_positions(&self, token_mint: &Pubkey) -> Result<Transaction> {
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

        Ok(self.create_transaction_builder(&to_close))
    }

    /// Deposit tokens into a lending pool position owned by a margin account in
    /// an ATA position.
    ///
    /// Figures out if needed, and uses if so:
    /// - adapter vs accounting invoke
    /// - create position
    /// - refresh position
    ///
    /// # Params
    ///
    /// `underlying_mint` - The address of the mint for the tokens being deposited
    /// `source` - The token account that the deposit will be transfered from,
    ///            defaults to ata of source authority.
    /// `change` - The amount of tokens to deposit
    /// `authority` - The owner of the source account
    pub async fn pool_deposit(
        &self,
        underlying_mint: &Pubkey,
        source: Option<Pubkey>,
        change: TokenChange,
        authority: MarginActionAuthority,
    ) -> Result<TransactionBuilder> {
        let target = self.pool_deposit_target(underlying_mint).await?;
        let source_authority = Some(authority.resolve(&self.ix));
        let tx = self
            .invoke_ctx()
            .pool_deposit(*underlying_mint, source, source_authority, target, change)
            .ijoin();
        Ok(self.sign(tx))
    }

    async fn pool_deposit_target(&self, underlying_mint: &Pubkey) -> Result<PoolTargetPosition> {
        let state = self.get_account_state().await?;
        PoolTargetPosition::new(
            &state,
            &MarginPoolIxBuilder::new(*underlying_mint).deposit_note_mint,
            &self.rpc.payer().pubkey(),
            async {
                self.get_pool(underlying_mint)
                    .await
                    .map(|p| p.token_price_oracle)
            },
        )
        .await
    }

    /// DEPRECATED: use pool_deposit instead
    ///
    /// this uses the old style of registering positions (non-ata) which will
    /// stop being supported.
    pub async fn pool_deposit_deprecated(
        &self,
        token_mint: &Pubkey,
        source: Option<Pubkey>,
        change: TokenChange,
        authority: MarginActionAuthority,
    ) -> Result<TransactionBuilder> {
        let mut instructions = vec![];
        let authority = authority.resolve(&self.ix);
        let source = source.or_ata(&authority, token_mint);
        let pool = MarginPoolIxBuilder::new(*token_mint);
        let (position, maybe_create) = self.get_or_create_position(&pool.deposit_note_mint).await?;
        let inner_ix = pool.deposit(authority, source, position, change);
        if let Some(create) = maybe_create {
            instructions.push(create);
            if self.ix.needs_signature(&inner_ix) {
                instructions.push(self.refresh_pool_position(token_mint).await?);
            }
        }
        instructions.push(self.smart_invoke(inner_ix));

        Ok(self.create_transaction_builder(&instructions))
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
        let token_metadata = get_token_metadata(&self.rpc, token_mint).await?;

        let deposit_position = self
            .get_or_push_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let _ = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_refresh_loan_ix =
            pool.margin_refresh_position(self.ix.address, token_metadata.pyth_price);
        instructions.push(self.ix.accounting_invoke(inner_refresh_loan_ix));

        let inner_borrow_ix = pool.margin_borrow(self.ix.address, deposit_position, change);

        instructions.push(self.adapter_invoke_ix(inner_borrow_ix));
        Ok(self.create_transaction_builder(&instructions))
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
            .get_or_push_create_position(&mut instructions, &pool.deposit_note_mint)
            .await?;
        let _ = self
            .get_or_create_pool_loan_position(&mut instructions, &pool)
            .await?;

        let inner_repay_ix = pool.margin_repay(self.ix.address, deposit_position, change);

        instructions.push(self.adapter_invoke_ix(inner_repay_ix));
        Ok(self.create_transaction_builder(&instructions))
    }

    /// Repay a loan from a token account of the underlying
    ///
    /// # Params
    ///
    /// `token_mint` - The address of the mint for the tokens that were borrowed
    /// `source` - Token account where funds originate, defaults to authority's ATA
    /// `change` - The amount of tokens to repay
    /// `authority` - The margin account who owns the loan and the tokens to repay
    pub fn pool_repay(
        &self,
        token_mint: Pubkey,
        source: Option<Pubkey>,
        change: TokenChange,
        authority: MarginActionAuthority,
    ) -> TransactionBuilder {
        let authority = authority.resolve(&self.ix);
        let source = source.or_ata(&authority, &token_mint);
        let pool = MarginPoolIxBuilder::new(token_mint);
        let loan_notes = derive_loan_account(&self.ix.address, &pool.loan_note_mint);
        let inner_ix = pool.repay(authority, source, loan_notes, change);
        let wrapped_ix = self.smart_invoke(inner_ix);

        self.create_transaction_builder(&[wrapped_ix])
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
            .get_or_push_create_position(&mut instructions, &pool.deposit_note_mint)
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
        target_token_mint: &Pubkey,
        swap_pool: &Pubkey,
        pool_mint: &Pubkey,
        fee_account: &Pubkey,
        source_token_account: &Pubkey,
        destination_token_account: &Pubkey,
        swap_program: &Pubkey,
        // for levswap
        change: TokenChange,
        minimum_amount_out: u64,
    ) -> Result<Vec<TransactionBuilder>> {
        let source_pool = MarginPoolIxBuilder::new(*source_token_mint);
        let source = self
            .get_account_state()
            .await?
            .position_address(&source_pool.deposit_note_mint)?;
        let target = self.pool_deposit_target(target_token_mint).await?;
        let tx = self.invoke_ctx().swap(
            SplSwap {
                program: *swap_program,
                address: *swap_pool,
                pool_mint: *pool_mint,
                token_a: *source_token_mint,
                token_b: *target_token_mint,
                token_a_vault: *source_token_account,
                token_b_vault: *destination_token_account,
                fee_account: *fee_account,
            },
            Some(source),
            target,
            change,
            minimum_amount_out,
        );
        Ok(self.sign(tx))
    }

    fn sign<Tx: WithSigner>(&self, tx: Tx) -> Tx::Output {
        match self.signer.as_ref() {
            Some(signer) => tx.with_signer(signer.clone()),
            None => tx.without_signer(),
        }
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

        let mut transactions = vec![];
        let setup_instructions = self.setup_swap(builder).await?;
        if !setup_instructions.is_empty() {
            let setup = self.create_transaction_builder(&setup_instructions);
            transactions.push(setup);
        }

        transactions.push(self.create_transaction_builder(&[
            ComputeBudgetInstruction::set_compute_unit_limit(800000),
            self.adapter_invoke_ix(inner_swap_ix),
        ]));

        Ok(transactions)
    }

    async fn setup_swap(&self, builder: &MarginSwapRouteIxBuilder) -> Result<Vec<Instruction>> {
        let mut setup_instructions = vec![];
        for deposit_note_mint in builder.get_pool_note_mints() {
            self.get_or_push_create_position(&mut setup_instructions, deposit_note_mint)
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
        txs.signers
            .push(self.signer.as_ref().context("missing signer")?.clone());

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

    /// Verify that the margin account is unhealthy
    pub async fn verify_unhealthy(&self) -> Result<Transaction> {
        self.create_unsigned_transaction(&[self.ix.verify_unhealthy()])
            .await
    }

    /// Refresh a user's position in a margin pool
    pub async fn refresh_pool_position(&self, token_mint: &Pubkey) -> Result<Instruction> {
        let ix_builder = MarginPoolIxBuilder::new(*token_mint);
        let pool_oracle = self.get_pool(token_mint).await?.token_price_oracle;

        Ok(self
            .ix
            .accounting_invoke(ix_builder.margin_refresh_position(self.ix.address, pool_oracle)))
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
                    .with_signers(self.signers())
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
    ) -> Result<TransactionBuilder> {
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

        Ok(self.create_transaction_builder(&instructions))
    }

    /// Get the latest [MarginAccount] state
    pub async fn get_account_state(&self) -> Result<Box<MarginAccount>> {
        Ok(Box::new(
            get_margin_account(&self.rpc, &self.ix.address).await?,
        ))
    }

    /// Append instructions to refresh pool positions to instructions
    pub async fn refresh_all_pool_positions_underlying_to_tx(
        &self,
    ) -> Result<HashMap<Pubkey, TransactionBuilder>> {
        let state = self.get_account_state().await?;
        refresh_all_pool_positions_underlying_to_tx(&self.rpc, &state).await
    }

    /// Append instructions to refresh deposit positions
    pub async fn refresh_deposit_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let state = self.get_account_state().await?;
        refresh_deposit_positions(&self.rpc, &state).await
    }

    /// Create an open orders account
    pub fn create_openbook_open_orders(
        &self,
        market: &Pubkey,
        program: &Pubkey,
    ) -> TransactionBuilder {
        let (open_orders_ix, _) =
            create_open_orders(*self.address(), *market, self.rpc.payer().pubkey(), program);
        let instruction = self.adapter_invoke_ix(open_orders_ix);
        self.create_transaction_builder(&[instruction])
    }

    /// Close an open orders account
    pub fn close_openbook_open_orders(
        &self,
        market: &Pubkey,
        program: &Pubkey,
    ) -> TransactionBuilder {
        let open_orders_ix =
            close_open_orders(*self.address(), *market, self.rpc.payer().pubkey(), program);
        let instruction = self.adapter_invoke_ix(open_orders_ix);
        self.create_transaction_builder(&[instruction])
    }

    async fn get_pool(&self, token_mint: &Pubkey) -> Result<MarginPool> {
        let pool_builder = MarginPoolIxBuilder::new(*token_mint);
        let account = self
            .rpc
            .get_account(&pool_builder.address)
            .await?
            .context("could not find pool")?;

        Ok(MarginPool::try_deserialize(&mut &account.data[..])?)
    }

    async fn get_or_push_create_position(
        &self,
        instructions: &mut Vec<Instruction>,
        token_mint: &Pubkey,
    ) -> Result<Pubkey> {
        let (address, create) = self.get_or_create_position(token_mint).await?;
        if let Some(ix) = create {
            instructions.push(ix);
        }
        Ok(address)
    }

    async fn get_or_create_position(
        &self,
        token_mint: &Pubkey,
    ) -> Result<(Pubkey, Option<Instruction>)> {
        match self.get_position_token_account(token_mint).await? {
            Some(address) => Ok((address, None)),
            None => Ok((
                derive_position_token_account(&self.ix.address, token_mint),
                Some(self.ix.register_position(*token_mint)),
            )),
        }
    }

    async fn get_position_token_account(&self, token_mint: &Pubkey) -> Result<Option<Pubkey>> {
        Ok(self
            .get_account_state()
            .await?
            .positions()
            .find(|p| p.token == *token_mint)
            .map(|p| p.address))
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

    /// If the margin account needs to sign, then use adapter or liquidator
    /// invoke, otherwise use accounting invoke.
    pub fn smart_invoke(&self, inner: Instruction) -> Instruction {
        if self.ix.needs_signature(&inner) {
            self.adapter_invoke_ix(inner)
        } else {
            self.ix.accounting_invoke(inner)
        }
    }
}

/// Instructions invoked through a margin account may require a signer that
/// could potentially be any account, depending on the situation. For example, a
/// deposit into the margin account requires a signer from the source account,
/// which could be anyone.
///
/// Most cases follow one of a few common patterns though. For example the
/// margin account authority or the margin account itself is most likely to be
/// the account authorizing a deposit. But in theory it could be anyone.
///
/// Rather than requiring the caller to always specify the address of the
/// authority of this action, we can leverage some of the data that is already
/// encapsulated within the MarginIxBuilder. So the caller of the function can
/// just specify that it wants to use a concept, such as "authority", rather
/// than having to struggle to identify the authority.
pub enum MarginActionAuthority {
    /// - The builder's configured "authority" for the margin account.
    /// - Typically, the acccount owner or its liquidator, depending on context.
    /// - See method: `MarginIxBuilder::authority()`.
    /// - In theory, this is *expected* to be whatever the actual MarginAccount
    ///   on chain is configured to require as the authority for user actions,
    ///   but there is nothing in MarginIxBuilder that guarantees its
    ///   "authority" is consistent with the on-chain state.
    AccountAuthority,
    /// The margin account itself is the authority, so there is no external
    /// signature needed.
    MarginAccount,
    /// Some other account that the tx_builder doesn't know about needs to sign.
    AdHoc(Pubkey),
}

impl MarginActionAuthority {
    fn resolve(self, ixb: &MarginIxBuilder) -> Pubkey {
        match self {
            MarginActionAuthority::AccountAuthority => ixb.authority(),
            MarginActionAuthority::MarginAccount => ixb.address,
            MarginActionAuthority::AdHoc(adhoc) => adhoc,
        }
    }
}

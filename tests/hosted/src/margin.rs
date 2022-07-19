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

use std::collections::HashMap;
use std::sync::Arc;

use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas};
use anyhow::{bail, Error};

use jet_margin::{MarginAccount, PositionKind};
use jet_margin_sdk::ix_builder::{ControlIxBuilder, MarginPoolConfiguration, MarginPoolIxBuilder};
use jet_margin_sdk::swap::SwapPool;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_program;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};

use jet_control::TokenMetadataParams;
use jet_margin_pool::{Amount, MarginPool, MarginPoolConfig, TokenChange};
use jet_margin_sdk::tx_builder::MarginTxBuilder;
use jet_metadata::{LiquidatorMetadata, MarginAdapterMetadata, TokenKind, TokenMetadata};
use jet_simulation::{send_and_confirm, solana_rpc_api::SolanaRpcClient};

use crate::tokens::TokenOracle;

/// Information needed to create a new margin pool
pub struct MarginPoolSetupInfo {
    pub token: Pubkey,
    pub token_kind: TokenKind,
    pub collateral_weight: u16,
    pub max_leverage: u16,
    pub oracle: TokenOracle,
    pub config: MarginPoolConfig,
}

/// Utility for making use of the Jet margin system.
pub struct MarginClient {
    rpc: Arc<dyn SolanaRpcClient>,
}

impl MarginClient {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self { rpc }
    }

    pub async fn user(&self, keypair: &Keypair, seed: u16) -> Result<MarginUser, Error> {
        let tx = MarginTxBuilder::new(
            self.rpc.clone(),
            Some(Keypair::from_bytes(&keypair.to_bytes())?),
            keypair.pubkey(),
            seed,
        );

        Ok(MarginUser {
            tx,
            rpc: self.rpc.clone(),
        })
    }

    pub async fn liquidator(
        &self,
        keypair: &Keypair,
        owner: &Pubkey,
        seed: u16,
    ) -> Result<MarginUser, Error> {
        let tx = MarginTxBuilder::new_liquidator(
            self.rpc.clone(),
            Some(Keypair::from_bytes(&keypair.to_bytes())?),
            *owner,
            seed,
            keypair.pubkey(),
        );

        Ok(MarginUser {
            tx,
            rpc: self.rpc.clone(),
        })
    }

    /// Find all the margin pools created
    pub async fn find_pools(&self) -> Result<Vec<MarginPool>, Error> {
        self.rpc
            .get_program_accounts(
                &jet_margin_pool::ID,
                Some(std::mem::size_of::<MarginPool>()),
            )
            .await?
            .into_iter()
            .map(|(_, account)| {
                MarginPool::try_deserialize(&mut &account.data[..]).map_err(Error::from)
            })
            .collect()
    }

    pub async fn get_pool(&self, token: &Pubkey) -> Result<MarginPool, Error> {
        let pool_builder = MarginPoolIxBuilder::new(*token);
        let account = self.rpc.get_account(&pool_builder.address).await?;

        if account.is_none() {
            bail!("could not find pool");
        }

        MarginPool::try_deserialize(&mut &account.unwrap().data[..]).map_err(Error::from)
    }

    pub async fn create_authority(&self) -> Result<(), Error> {
        let ix = ControlIxBuilder::new(self.rpc.payer().pubkey()).create_authority();

        send_and_confirm(&self.rpc, &[ix], &[]).await?;
        Ok(())
    }

    pub async fn register_adapter(&self, adapter: &Pubkey) -> Result<(), Error> {
        let ix = ControlIxBuilder::new(self.rpc.payer().pubkey()).register_adapter(adapter);

        send_and_confirm(&self.rpc, &[ix], &[]).await?;
        Ok(())
    }

    pub async fn configure_margin_pool(
        &self,
        token: &Pubkey,
        config: &MarginPoolConfiguration,
    ) -> Result<(), Error> {
        let ix =
            ControlIxBuilder::new(self.rpc.payer().pubkey()).configure_margin_pool(token, config);

        send_and_confirm(&self.rpc, &[ix], &[]).await?;

        Ok(())
    }

    /// Create a new margin pool for a token
    pub async fn create_pool(&self, setup_info: &MarginPoolSetupInfo) -> Result<(), Error> {
        let ix =
            ControlIxBuilder::new(self.rpc.payer().pubkey()).create_margin_pool(&setup_info.token);

        send_and_confirm(&self.rpc, &[ix], &[]).await?;

        self.configure_margin_pool(
            &setup_info.token,
            &MarginPoolConfiguration {
                pyth_price: Some(setup_info.oracle.price),
                pyth_product: Some(setup_info.oracle.product),
                metadata: Some(TokenMetadataParams {
                    token_kind: TokenKind::Collateral,
                    collateral_weight: setup_info.collateral_weight,
                    max_leverage: setup_info.max_leverage,
                }),
                parameters: Some(setup_info.config.clone()),
            },
        )
        .await?;

        Ok(())
    }

    pub async fn set_liquidator_metadata(
        &self,
        liquidator: Pubkey,
        is_liquidator: bool,
    ) -> Result<(), Error> {
        let ix = ControlIxBuilder::new(self.rpc.payer().pubkey())
            .set_liquidator(&liquidator, is_liquidator);

        send_and_confirm(&self.rpc, &[ix], &[]).await?;

        Ok(())
    }

    pub async fn get_account(&self, address: &Pubkey) -> Result<Box<MarginAccount>, Error> {
        let account_data = self.rpc.get_account(address).await?;

        match account_data {
            None => bail!("no margin account found {}", address),
            Some(account) => Ok(Box::new(MarginAccount::try_deserialize(
                &mut &account.data[..],
            )?)),
        }
    }
}

pub struct MarginUser {
    tx: MarginTxBuilder,
    rpc: Arc<dyn SolanaRpcClient>,
}

impl MarginUser {
    pub async fn print(&self) {
        println!("{:#?}", self.tx.get_account_state().await.unwrap())
    }
    async fn send_confirm_tx(&self, tx: Transaction) -> Result<(), Error> {
        let _ = self.rpc.send_and_confirm_transaction(&tx).await?;
        Ok(())
    }

    async fn send_confirm_all_tx(
        &self,
        transactions: impl IntoIterator<Item = Transaction>,
    ) -> Result<(), Error> {
        futures::future::join_all(transactions.into_iter().map(|tx| self.send_confirm_tx(tx)))
            .await
            .into_iter()
            .collect()
    }
}

impl MarginUser {
    pub fn owner(&self) -> &Pubkey {
        self.tx.owner()
    }

    pub fn signer(&self) -> Pubkey {
        self.tx.signer()
    }

    pub fn address(&self) -> &Pubkey {
        self.tx.address()
    }

    pub fn seed(&self) -> u16 {
        self.tx.seed()
    }

    pub async fn create_account(&self) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.create_account().await?).await
    }

    /// Close the margin account
    ///
    /// # Error
    ///
    /// Returns an error if the account is not empty, in which case positions
    /// should be closed first.
    pub async fn close_account(&self) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.close_account().await?).await
    }

    pub async fn refresh_pool_position(&self, token_mint: &Pubkey) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.refresh_pool_position(token_mint).await?)
            .await
    }

    pub async fn refresh_all_pool_positions(&self) -> Result<(), Error> {
        self.send_confirm_all_tx(self.tx.refresh_all_pool_positions().await?)
            .await
    }

    pub async fn refresh_all_position_metadata(&self) -> Result<(), Error> {
        self.send_confirm_all_tx(self.tx.refresh_all_position_metadata().await?)
            .await
    }

    pub async fn deposit(
        &self,
        mint: &Pubkey,
        source: &Pubkey,
        change: TokenChange,
    ) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.deposit(mint, source, change).await?)
            .await
    }

    pub async fn withdraw(
        &self,
        mint: &Pubkey,
        destination: &Pubkey,
        change: TokenChange,
    ) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.withdraw(mint, destination, change).await?)
            .await
    }

    pub async fn borrow(&self, mint: &Pubkey, change: TokenChange) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.borrow(mint, change).await?)
            .await
    }

    pub async fn margin_repay(&self, mint: &Pubkey, change: TokenChange) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.margin_repay(mint, change).await?)
            .await
    }

    pub async fn repay(
        &self,
        mint: &Pubkey,
        source: &Pubkey,
        change: TokenChange,
    ) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.repay(mint, source, change).await?)
            .await
    }

    /// Swap between two tokens using a swap pool.
    ///
    /// The `source_mint` and `destination_mint` determine the direction of
    /// the swap.
    #[allow(clippy::too_many_arguments)]
    pub async fn swap(
        &self,
        program_id: &Pubkey,
        source_mint: &Pubkey,
        destination_mint: &Pubkey,
        transit_source_account: &Pubkey,
        transit_destination_account: &Pubkey,
        swap_pool: &SwapPool,
        amount_in: Amount,
        minimum_amount_out: Amount,
    ) -> Result<(), Error> {
        // Determine the order of token_a and token_b based on direction of swap
        let (source_token, destination_token) = if source_mint == &swap_pool.mint_a {
            (&swap_pool.token_a, &swap_pool.token_b)
        } else {
            (&swap_pool.token_b, &swap_pool.token_a)
        };
        self.send_confirm_tx(
            self.tx
                .swap(
                    source_mint,
                    destination_mint,
                    transit_source_account,
                    transit_destination_account,
                    &swap_pool.pool,
                    &swap_pool.pool_mint,
                    &swap_pool.fee_account,
                    source_token,
                    destination_token,
                    program_id,
                    amount_in,
                    minimum_amount_out,
                )
                .await?,
        )
        .await
    }

    pub async fn liquidate_begin(&self, refresh_positions: bool) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.liquidate_begin(refresh_positions).await?)
            .await
    }

    pub async fn liquidate_end(&self, original_liquidator: Option<Pubkey>) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.liquidate_end(original_liquidator).await?)
            .await
    }

    pub async fn verify_healthy(&self) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.verify_healthy().await?).await
    }

    /// Close a user's empty positions.
    pub async fn close_empty_positions(
        &self,
        loan_to_token: &HashMap<Pubkey, Pubkey>,
    ) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.close_empty_positions(loan_to_token).await?)
            .await
    }

    /// Close a user's token positions for a specific mint.
    pub async fn close_token_positions(&self, token_mint: &Pubkey) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.close_token_positions(token_mint).await?)
            .await
    }

    /// Close a user's token position for a mint, with the specified and token kind.
    pub async fn close_token_position(
        &self,
        token_mint: &Pubkey,
        kind: PositionKind,
    ) -> Result<(), Error> {
        self.send_confirm_tx(self.tx.close_pool_position(token_mint, kind).await?)
            .await
    }
}

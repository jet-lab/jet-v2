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

use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;

use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{
    RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
};
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::account::Account;
use solana_sdk::clock::Clock;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::slot_history::Slot;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::TransactionStatus;

/// Represents some client interface to the Solana network.
#[async_trait]
pub trait SolanaRpcClient: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>>;
    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>>;
    async fn get_genesis_hash(&self) -> Result<Hash>;
    async fn get_latest_blockhash(&self) -> Result<Hash>;
    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> Result<u64>;
    async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature>;
    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Option<TransactionStatus>>>;

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> Result<Vec<(Pubkey, Account)>>;

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> Result<()>;

    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let signature = self.send_transaction(transaction).await?;
        let _ = self.confirm_transactions(&[signature]).await?;

        Ok(signature)
    }

    async fn confirm_transactions(&self, signatures: &[Signature]) -> Result<Vec<bool>> {
        for _ in 0..7 {
            let statuses = self.get_signature_statuses(signatures).await?;

            if statuses.iter().all(|s| s.is_some()) {
                return Ok(statuses
                    .into_iter()
                    .map(|s| s.unwrap().err.is_none())
                    .collect());
            }
        }

        bail!("failed to confirm signatures: {:?}", signatures);
    }

    async fn create_transaction(
        &self,
        signers: &[&Keypair],
        instructions: &[Instruction],
    ) -> Result<Transaction> {
        let blockhash = self.get_latest_blockhash().await?;
        let mut all_signers = vec![self.payer()];

        all_signers.extend(signers);

        Ok(Transaction::new_signed_with_payer(
            instructions,
            Some(&self.payer().pubkey()),
            &all_signers,
            blockhash,
        ))
    }
    async fn get_slot(&self, commitment_config: Option<CommitmentConfig>) -> Result<Slot>;

    async fn get_clock(&self) -> Result<Clock>;
    async fn set_clock(&self, new_clock: Clock) -> Result<()>;

    fn payer(&self) -> &Keypair;
}

pub struct RpcConnection(Arc<RpcContext>);

struct RpcContext {
    rpc: RpcClient,
    payer: Keypair,
    tx_config: Option<RpcSendTransactionConfig>,
}

impl RpcConnection {
    pub fn new(payer: Keypair, rpc: RpcClient) -> RpcConnection {
        RpcConnection(Arc::new(RpcContext {
            rpc,
            payer,
            tx_config: None,
        }))
    }

    pub fn new_with_config(
        payer: Keypair,
        rpc: RpcClient,
        tx_config: Option<RpcSendTransactionConfig>,
    ) -> RpcConnection {
        RpcConnection(Arc::new(RpcContext {
            rpc,
            payer,
            tx_config,
        }))
    }

    /// Optimistic = assume there is no risk. so we don't need:
    /// - finality (processed can be trusted)
    /// - preflight checks (not worried about losing sol)
    ///
    /// This is desirable for testing because:
    /// - tests can run faster (never need to wait for finality)
    /// - validator logs are more comprehensive (preflight checks obscure error logs)
    /// - there is nothing at stake in a local test validator
    pub fn new_optimistic(payer: Keypair, url: &str) -> RpcConnection {
        RpcConnection(Arc::new(RpcContext {
            rpc: RpcClient::new_with_commitment(
                url.to_owned(),
                CommitmentConfig {
                    commitment: CommitmentLevel::Processed,
                },
            ),
            payer,
            tx_config: Some(solana_client::rpc_config::RpcSendTransactionConfig {
                skip_preflight: true,
                ..Default::default()
            }),
        }))
    }

    /// Get the underlying [RpcClient]
    pub fn client(&self) -> &RpcClient {
        &self.0.rpc
    }

    /// Get the underlying transaction config
    pub fn tx_config(&self) -> Option<&RpcSendTransactionConfig> {
        self.0.tx_config.as_ref()
    }
}

#[async_trait]
impl SolanaRpcClient for RpcConnection {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let commitment = self.0.rpc.commitment();
        let tx_config = self.0.tx_config.unwrap_or(RpcSendTransactionConfig {
            preflight_commitment: Some(commitment.commitment),
            ..Default::default()
        });

        Ok(self
            .0
            .rpc
            .send_and_confirm_transaction_with_spinner_and_config(
                transaction,
                commitment,
                tx_config,
            )
            .await?)
    }

    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>> {
        Ok(self
            .0
            .rpc
            .get_multiple_accounts(&[*address])
            .await?
            .pop()
            .unwrap())
    }

    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        Ok(self.0.rpc.get_multiple_accounts(pubkeys).await?)
    }

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let filters = size.map(|s| vec![RpcFilterType::DataSize(s as u64)]);

        Ok(self
            .0
            .rpc
            .get_program_accounts_with_config(
                program_id,
                RpcProgramAccountsConfig {
                    filters,
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64Zstd),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await?)
    }

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> Result<()> {
        self.0.rpc.request_airdrop(account, amount).await?;

        Ok(())
    }

    async fn get_genesis_hash(&self) -> Result<Hash> {
        let hash = self.0.rpc.get_genesis_hash().await?;

        Ok(hash)
    }

    async fn get_latest_blockhash(&self) -> Result<Hash> {
        let blockhash = self.0.rpc.get_latest_blockhash().await?;

        Ok(blockhash)
    }

    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> Result<u64> {
        Ok(self
            .0
            .rpc
            .get_minimum_balance_for_rent_exemption(length)
            .await?)
    }

    async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        Ok(self.0.rpc.send_transaction(transaction).await?)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Option<TransactionStatus>>> {
        Ok(self.0.rpc.get_signature_statuses(signatures).await?.value)
    }

    async fn get_clock(&self) -> Result<Clock> {
        let slot = self.0.rpc.get_slot().await?;
        let unix_timestamp = self.0.rpc.get_block_time(slot).await?;

        Ok(Clock {
            slot,
            unix_timestamp,
            ..Default::default() // epoch probably doesn't matter?
        })
    }

    async fn get_slot(&self, commitment_config: Option<CommitmentConfig>) -> Result<Slot> {
        match commitment_config {
            Some(commitment_config) => Ok(self
                .0
                .rpc
                .get_slot_with_commitment(commitment_config)
                .await?),
            None => Ok(self.0.rpc.get_slot().await?),
        }
    }

    async fn set_clock(&self, _new_clock: Clock) -> Result<()> {
        Ok(())
    }

    fn payer(&self) -> &Keypair {
        &self.0.payer
    }
}

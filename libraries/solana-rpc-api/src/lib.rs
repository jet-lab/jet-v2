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

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Result};
use async_trait::async_trait;

use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::account::Account;
use solana_sdk::clock::Clock;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::TransactionStatus;

/// Represents some client interface to the Solana network.
#[async_trait]
pub trait SolanaRpcClient: Send + Sync {
    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>>;
    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>>;
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

    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let signature = self.send_transaction(transaction).await?;
        let _ = self.confirm_transactions(&[signature]).await?;

        Ok(signature)
    }

    async fn confirm_transactions(&self, signatures: &[Signature]) -> Result<Vec<bool>> {
        for _ in 0..7 {
            let statuses = self.get_signature_statuses(signatures).await?;

            if !statuses.iter().all(|s| s.is_some()) {
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

    fn payer(&self) -> &Keypair;
    fn get_clock(&self) -> Option<Clock>;
    fn set_clock(&self, new_clock: Clock);
}

pub struct RpcConnection(Arc<RpcContext>);

struct RpcContext {
    rpc: RpcClient,
    payer: Keypair,
    blockhash: Mutex<(Hash, SystemTime)>,
}

impl RpcConnection {
    pub fn new(payer: Keypair, rpc: RpcClient) -> RpcConnection {
        RpcConnection(Arc::new(RpcContext {
            rpc,
            payer,
            blockhash: Mutex::new((Hash::new_unique(), SystemTime::now())),
        }))
    }
}

#[async_trait]
impl SolanaRpcClient for RpcConnection {
    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>> {
        let ctx = self.0.clone();
        let address = *address;

        Ok(tokio::task::spawn_blocking(move || {
            ctx.rpc
                .get_multiple_accounts(&[address])
                .map(|mut list| list.pop().unwrap())
        })
        .await??)
    }

    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        let ctx = self.0.clone();
        let pubkeys = pubkeys.to_vec();

        Ok(tokio::task::spawn_blocking(move || ctx.rpc.get_multiple_accounts(&pubkeys)).await??)
    }

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let ctx = self.0.clone();
        let program_id = *program_id;
        let filters = size.map(|s| vec![RpcFilterType::DataSize(s as u64)]);

        Ok(tokio::task::spawn_blocking(move || {
            ctx.rpc.get_program_accounts_with_config(
                &program_id,
                RpcProgramAccountsConfig {
                    filters,
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64Zstd),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        })
        .await??)
    }

    async fn get_latest_blockhash(&self) -> Result<Hash> {
        let (mut blockhash, retrieved_at) = *self.0.blockhash.lock().unwrap();

        if Duration::from_secs(1) < SystemTime::now().duration_since(retrieved_at).unwrap() {
            let ctx = self.0.clone();
            blockhash =
                tokio::task::spawn_blocking(move || ctx.rpc.get_latest_blockhash()).await??;
        }

        Ok(blockhash)
    }

    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> Result<u64> {
        let ctx = self.0.clone();

        Ok(tokio::task::spawn_blocking(move || {
            ctx.rpc.get_minimum_balance_for_rent_exemption(length)
        })
        .await??)
    }

    async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let ctx = self.0.clone();
        let tx = transaction.clone();

        Ok(tokio::task::spawn_blocking(move || ctx.rpc.send_transaction(&tx)).await??)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Option<TransactionStatus>>> {
        let ctx = self.0.clone();
        let sigs = signatures.to_vec();

        Ok(
            tokio::task::spawn_blocking(move || ctx.rpc.get_signature_statuses(&sigs))
                .await??
                .value,
        )
    }

    fn payer(&self) -> &Keypair {
        &self.0.payer
    }

    fn get_clock(&self) -> Option<Clock> {
        None
    }

    fn set_clock(&self, _new_clock: Clock) {}
}

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

use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;

use solana_sdk::account::Account;
use solana_sdk::clock::Clock;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::slot_history::Slot;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use jet_solana_client::rpc::{AccountFilter, SolanaRpc, SolanaRpcExtra, TransactionStatus};

use crate::runtime::TestRuntimeRpcClient;

/// Represents some client interface to the Solana network.
#[async_trait]
pub trait SolanaRpcClient: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_with_payer(&self, payer: Keypair) -> Box<dyn SolanaRpcClient>;
    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>>;
    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>>;
    async fn get_genesis_hash(&self) -> Result<Hash>;
    async fn get_latest_blockhash(&self) -> Result<Hash>;
    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> Result<u64>;
    async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature>;
    async fn send_versioned_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature>;
    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Option<TransactionStatus>>>;

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Vec<AccountFilter>,
    ) -> Result<Vec<(Pubkey, Account)>>;

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> Result<()>;

    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let signature = self.send_transaction(transaction).await?;
        let mut statuses = self.confirm_transactions(&[signature]).await?;

        if let Some(err) = statuses.pop().unwrap().status.err() {
            return Err(err).context(format!("Transaction error for {signature}"));
        }

        Ok(signature)
    }

    async fn send_and_confirm_versioned_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature> {
        let signature = self.send_versioned_transaction(transaction).await?;
        let mut statuses = self.confirm_transactions(&[signature]).await?;

        if let Some(err) = statuses.pop().unwrap().status.err() {
            return Err(err).context(format!("Transaction error for {signature}"));
        }

        Ok(signature)
    }

    async fn confirm_transactions(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<TransactionStatus>>;

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
    async fn wait_for_next_block(&self) -> Result<()>;

    fn payer(&self) -> &Keypair;
}

#[async_trait]
impl<Rpc> SolanaRpcClient for (Rpc, Keypair)
where
    Rpc: SolanaRpc + Send + Sync + Clone + 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        &self.0 as &dyn std::any::Any
    }

    fn clone_with_payer(&self, payer: Keypair) -> Box<dyn SolanaRpcClient> {
        Box::new((self.0.clone(), payer))
    }

    async fn get_account(&self, address: &Pubkey) -> Result<Option<Account>> {
        Ok(self.0.get_account(address).await?)
    }

    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        Ok(self.0.get_multiple_accounts(pubkeys).await?)
    }

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Vec<AccountFilter>,
    ) -> Result<Vec<(Pubkey, Account)>> {
        Ok(self.0.get_program_accounts(program_id, &filters).await?)
    }

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> Result<()> {
        self.0.airdrop(account, amount).await?;

        Ok(())
    }

    async fn confirm_transactions(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<TransactionStatus>> {
        Ok(self.0.confirm_transactions(signatures).await?)
    }

    async fn get_genesis_hash(&self) -> Result<Hash> {
        let hash = self.0.get_genesis_hash().await?;

        Ok(hash)
    }

    async fn get_latest_blockhash(&self) -> Result<Hash> {
        let blockhash = self.0.get_latest_blockhash().await?;

        Ok(blockhash)
    }

    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> Result<u64> {
        let rent = Rent::default();
        Ok(rent.minimum_balance(length))
    }

    async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        Ok(self.0.send_transaction_legacy(transaction).await?)
    }

    async fn send_versioned_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Signature> {
        Ok(self.0.send_transaction(transaction).await?)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Option<TransactionStatus>>> {
        Ok(self.0.get_signature_statuses(signatures).await?)
    }

    async fn get_clock(&self) -> Result<Clock> {
        let slot = self.0.get_slot().await?;
        let unix_timestamp = self.0.get_block_time(slot).await?;

        Ok(Clock {
            slot,
            unix_timestamp,
            ..Default::default() // epoch probably doesn't matter?
        })
    }

    async fn get_slot(&self, _commitment_config: Option<CommitmentConfig>) -> Result<Slot> {
        Ok(self.0.get_slot().await?)
    }

    async fn set_clock(&self, new_clock: Clock) -> Result<()> {
        if let Some(rpc) = self.as_any().downcast_ref::<TestRuntimeRpcClient>() {
            rpc.set_clock(&new_clock);
        }

        Ok(())
    }

    async fn wait_for_next_block(&self) -> Result<()> {
        if let Some(rpc) = self.as_any().downcast_ref::<TestRuntimeRpcClient>() {
            rpc.next_block();
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(400)).await;
        Ok(())
    }

    fn payer(&self) -> &Keypair {
        &self.1
    }
}

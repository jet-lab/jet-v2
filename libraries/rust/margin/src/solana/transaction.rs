/// TODO: this code might be better to put in solana-client or simulation, but
/// it's awkward to manage the dependencies.
use anyhow::Result;
use async_trait::async_trait;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::sync::Arc;

use crate::util::asynchronous::MapAsync;

pub use jet_solana_client::transaction::*; // TODO: remove

use super::keypair::clone_refs;

/// Implementers are expected to send a TransactionBuilder to a real or simulated solana network as a transaction
#[async_trait]
pub trait SendTransactionBuilder {
    /// Converts a TransactionBuilder to a Transaction,
    /// finalizing its set of instructions as the selection for the actual Transaction
    async fn compile(&self, tx: TransactionBuilder) -> Result<Transaction>;

    /// Sends the transaction unchanged
    async fn send_and_confirm(&self, transaction: TransactionBuilder) -> Result<Signature>;

    /// simple ad hoc transaction sender
    async fn send_and_confirm_1tx(
        &self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<Signature> {
        self.send_and_confirm(TransactionBuilder {
            instructions: instructions.to_vec(),
            signers: clone_refs(signers),
        })
        .await
    }

    /// Send, minimizing number of transactions - see `condense` doc
    /// sends transactions all at once
    async fn send_and_confirm_condensed(
        &self,
        transactions: Vec<TransactionBuilder>,
    ) -> Result<Vec<Signature>>;

    /// Send, minimizing number of transactions - see `condense` doc
    /// sends transactions one at a time after confirming the last
    async fn send_and_confirm_condensed_in_order(
        &self,
        transactions: Vec<TransactionBuilder>,
    ) -> Result<Vec<Signature>>;
}

#[async_trait]
impl SendTransactionBuilder for Arc<dyn SolanaRpcClient> {
    async fn compile(&self, tx: TransactionBuilder) -> Result<Transaction> {
        let signers = tx.signers.iter().collect::<Vec<&Keypair>>();
        self.create_transaction(&signers, &tx.instructions).await
    }

    async fn send_and_confirm(&self, tx: TransactionBuilder) -> Result<Signature> {
        self.send_and_confirm_transaction(&self.compile(tx).await?)
            .await
    }

    async fn send_and_confirm_condensed(
        &self,
        transactions: Vec<TransactionBuilder>,
    ) -> Result<Vec<Signature>> {
        condense(&transactions, &self.payer().pubkey())?
            .into_iter()
            .map_async(|tx| self.send_and_confirm(tx))
            .await
    }

    async fn send_and_confirm_condensed_in_order(
        &self,
        transactions: Vec<TransactionBuilder>,
    ) -> Result<Vec<Signature>> {
        condense(&transactions, &self.payer().pubkey())?
            .into_iter()
            .map_async_chunked(1, |tx| self.send_and_confirm(tx))
            .await
    }
}

/// Analogous to SendTransactionBuilder, but allows you to call it with the
/// TransactionBuilder as the receiver when it would enable a cleaner
/// method-chaining syntax.
#[async_trait]
pub trait TransactionBuilderExt {
    /// SendTransactionBuilder::compile
    async fn compile<C: SendTransactionBuilder + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Transaction>;

    /// SendTransactionBuilder::send_and_confirm
    async fn send_and_confirm<C: SendTransactionBuilder + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Signature>;
}

#[async_trait]
impl TransactionBuilderExt for TransactionBuilder {
    /// SendTransactionBuilder::compile
    async fn compile<C: SendTransactionBuilder + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Transaction> {
        client.compile(self).await
    }

    /// SendTransactionBuilder::send_and_confirm
    async fn send_and_confirm<C: SendTransactionBuilder + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Signature> {
        client.send_and_confirm(self).await
    }
}

/// Analogous to SendTransactionBuilder, but allows you to call it with the
/// Vec<TransactionBuilder> as the receiver when it would enable a cleaner
/// method-chaining syntax.
#[async_trait]
pub trait InverseSendTransactionBuilder {
    /// SendTransactionBuilder::send_and_confirm_condensed
    async fn send_and_confirm_condensed<C: SendTransactionBuilder + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>>;

    /// SendTransactionBuilder::send_and_confirm_condensed_in_order
    async fn send_and_confirm_condensed_in_order<C: SendTransactionBuilder + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>>;
}

#[async_trait]
impl InverseSendTransactionBuilder for Vec<TransactionBuilder> {
    async fn send_and_confirm_condensed<C: SendTransactionBuilder + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>> {
        client.send_and_confirm_condensed(self).await
    }

    async fn send_and_confirm_condensed_in_order<C: SendTransactionBuilder + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>> {
        client.send_and_confirm_condensed_in_order(self).await
    }
}

/// TODO: this code might be better to put in solana-client or simulation, but
/// it's awkward to manage the dependencies.
use anyhow::Result;
use async_trait::async_trait;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{
    instruction::Instruction, signature::Signature, signer::Signer, transaction::Transaction,
};
use std::{ops::Deref, sync::Arc};

use crate::util::asynchronous::MapAsync;

pub use jet_solana_client::transaction::*; // TODO: remove

/// Implementers are expected to send a TransactionBuilder to a real or simulated solana network as a transaction
#[async_trait]
pub trait SendTransactionBuilder<K> {
    /// Converts a TransactionBuilder to a Transaction,
    /// finalizing its set of instructions as the selection for the actual Transaction
    async fn compile(&self, tx: InstructionBundle<K>) -> Result<Transaction>;

    /// Sends the transaction unchanged
    async fn send_and_confirm(&self, transaction: InstructionBundle<K>) -> Result<Signature>;

    /// simple ad hoc transaction sender. use `flexify` if necessary to get a good
    /// input type.
    async fn send_and_confirm_1tx(
        &self,
        instructions: &[Instruction],
        signers: Vec<K>,
    ) -> Result<Signature>
    where
        K: Send + Sync + 'static,
        Self: SendTransactionBuilder<K>,
    {
        self.send_and_confirm(InstructionBundle {
            instructions: instructions.to_vec(),
            signers,
        })
        .await
    }

    /// Send, minimizing number of transactions - see `condense` doc
    /// sends transactions all at once
    /// TODO: rename this to indicate that it's not ordered
    async fn send_and_confirm_condensed(
        &self,
        transactions: Vec<InstructionBundle<K>>,
    ) -> Result<Vec<Signature>>;

    /// Send, minimizing number of transactions - see `condense` doc
    /// sends transactions one at a time after confirming the last
    async fn send_and_confirm_condensed_in_order(
        &self,
        transactions: Vec<InstructionBundle<K>>,
    ) -> Result<Vec<Signature>>;
}

#[async_trait]
impl<K, S> SendTransactionBuilder<K> for Arc<dyn SolanaRpcClient>
where
    K: FlexKey<Inner = S> + 'static,
    S: ?Sized + Signer,
{
    async fn compile(&self, tx: InstructionBundle<K>) -> Result<Transaction> {
        let blockhash = self.get_latest_blockhash().await?;
        Ok(tx.compile(self.payer(), blockhash)?)
    }

    async fn send_and_confirm(&self, tx: InstructionBundle<K>) -> Result<Signature> {
        self.send_and_confirm_transaction(&self.compile(tx).await?)
            .await
    }

    async fn send_and_confirm_condensed(
        &self,
        transactions: Vec<InstructionBundle<K>>,
    ) -> Result<Vec<Signature>> {
        condense(&transactions, &self.payer().pubkey())?
            .into_iter()
            .map_async(|tx| self.send_and_confirm(tx))
            .await
    }

    async fn send_and_confirm_condensed_in_order(
        &self,
        transactions: Vec<InstructionBundle<K>>,
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
pub trait TransactionBuilderExt<K> {
    /// SendTransactionBuilder::compile
    async fn compile<C: SendTransactionBuilder<K> + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Transaction>;

    /// SendTransactionBuilder::send_and_confirm
    async fn send_and_confirm<C: SendTransactionBuilder<K> + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Signature>;
}

#[async_trait]
impl<K, S> TransactionBuilderExt<K> for InstructionBundle<K>
where
    K: FlexKey<Inner = S>,
    S: ?Sized + Signer,
{
    /// SendTransactionBuilder::compile
    async fn compile<C: SendTransactionBuilder<K> + Send + Sync>(
        self,
        client: &C,
    ) -> Result<Transaction> {
        client.compile(self).await
    }

    /// SendTransactionBuilder::send_and_confirm
    async fn send_and_confirm<C: SendTransactionBuilder<K> + Send + Sync>(
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
pub trait InverseSendTransactionBuilder<K> {
    /// SendTransactionBuilder::send_and_confirm_condensed
    /// TODO: rename this to indicate that it's not ordered
    async fn send_and_confirm_condensed<C: SendTransactionBuilder<K> + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>>;

    /// SendTransactionBuilder::send_and_confirm_condensed_in_order
    async fn send_and_confirm_condensed_in_order<C: SendTransactionBuilder<K> + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>>;
}

#[async_trait]
impl<K, S> InverseSendTransactionBuilder<K> for Vec<InstructionBundle<K>>
where
    K: FlexKey<Inner = S>,
    S: ?Sized + Signer,
{
    async fn send_and_confirm_condensed<C: SendTransactionBuilder<K> + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>> {
        client.send_and_confirm_condensed(self).await
    }

    async fn send_and_confirm_condensed_in_order<C: SendTransactionBuilder<K> + Sync>(
        self,
        client: &C,
    ) -> Result<Vec<Signature>> {
        client.send_and_confirm_condensed_in_order(self).await
    }
}

/// This trait is used to simplify repetitive trait bounds. It encapsulates a
/// common collection of traits that are required for the trait implementations
/// in this file. Do not expand this trait to have additional trait bounds
/// unless you are certain that the additional trait bound is required in *all*
/// places where this is used as a trait bound.
///
/// A FlexSigner is a signer that...
///
/// has extra versatility to make it more useful:
/// - can be cloned
/// - is thread safe
///
/// is easier to construct:
/// - only needs to deref to a Signer, doesn't need to actually implement Signer
pub trait FlexKey: Deref<Target = Self::Inner> + Clone + Send + Sync {
    /// The Signer type that this Derefs to
    type Inner;
}
impl<S: Signer, F: Deref<Target = S> + Clone + Send + Sync> FlexKey for F {
    type Inner = S;
}

use anyhow::Result;
use async_trait::async_trait;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::hash::{Hash, HASH_BYTES};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::cmp::{max, min};
use std::sync::Arc;

use crate::util::data::DeepReverse;
use crate::{
    solana::keypair::clone_vec,
    util::{
        asynchronous::MapAsync,
        data::{Concat, Join},
    },
};

use super::keypair::clone_refs;

/// A group of instructions that are expected to be executed in the same transaction
/// Can be merged with other TransactionBuilder instances with `cat`, `concat`, or `ijoin`
#[derive(Debug, Default)]
pub struct TransactionBuilder {
    /// see above
    pub instructions: Vec<Instruction>,
    /// required for the included instructions, does not include a payer
    pub signers: Vec<Keypair>, //todo Arc<dyn Signer>
}

impl DeepReverse for TransactionBuilder {
    fn deep_reverse(mut self) -> Self {
        self.instructions.reverse();
        self
    }
}

impl Clone for TransactionBuilder {
    fn clone(&self) -> Self {
        Self {
            instructions: self.instructions.clone(),
            signers: clone_vec(&self.signers),
        }
    }
}

impl From<Vec<Instruction>> for TransactionBuilder {
    fn from(instructions: Vec<Instruction>) -> Self {
        TransactionBuilder {
            instructions,
            signers: vec![],
        }
    }
}

impl From<Instruction> for TransactionBuilder {
    fn from(ix: Instruction) -> Self {
        TransactionBuilder {
            instructions: vec![ix],
            signers: vec![],
        }
    }
}

impl TransactionBuilder {
    /// convert transaction to base64 string that would be submitted to rpc node
    pub fn encode(&self, hash: Hash, payer: &Keypair) -> Result<String> {
        let compiled = create_transaction(
            &self.signers.iter().collect::<Vec<&Keypair>>(),
            &self.instructions,
            hash,
            payer,
        )?;
        let serialized = bincode::serialize::<Transaction>(&compiled)?;
        Ok(base64::encode(serialized))
    }
}

impl Concat for TransactionBuilder {
    fn cat(mut self, other: Self) -> Self {
        self.instructions.extend(other.instructions.into_iter());
        self.signers.extend(other.signers.into_iter());

        Self { ..self }
    }

    fn cat_ref(mut self, other: &Self) -> Self {
        self.instructions
            .extend(other.instructions.clone().into_iter());
        self.signers.extend(clone_vec(&other.signers).into_iter());

        Self { ..self }
    }
}

/// Convert types to a TransactionBuilder while including signers. Serves a
/// similar purpose to From<Instruction>, but it's used when you also need to
/// add signers.
pub trait WithSigner: Sized {
    /// convert to a TransactionBuilder that includes this signer
    fn with_signer(self, signer: Keypair) -> TransactionBuilder {
        self.with_signers(&[signer])
    }
    /// convert to a TransactionBuilder that includes these signers
    fn with_signers(self, signers: &[Keypair]) -> TransactionBuilder;
}

impl WithSigner for Instruction {
    fn with_signers(self, signers: &[Keypair]) -> TransactionBuilder {
        vec![self].with_signers(signers)
    }
}

impl WithSigner for &[Instruction] {
    fn with_signers(self, signers: &[Keypair]) -> TransactionBuilder {
        TransactionBuilder {
            instructions: self.to_vec(),
            signers: clone_vec(signers),
        }
    }
}

impl WithSigner for TransactionBuilder {
    fn with_signers(mut self, signers: &[Keypair]) -> TransactionBuilder {
        self.signers.extend(clone_vec(signers));
        TransactionBuilder {
            instructions: self.instructions,
            signers: self.signers,
        }
    }
}

const MAX_TX_SIZE: usize = 1232;

/// Combines all the instructions within each of the TransactionBuilders into the smallest
///  possible number of TransactionBuilders that don't violate the rules:
/// - instructions that were already grouped in a TransactionBuilder must end up in the same TransactionBuilder
/// - transaction may not exceed size limit
/// - instructions order is not modified
pub fn condense(txs: &[TransactionBuilder], payer: &Keypair) -> Result<Vec<TransactionBuilder>> {
    condense_right(txs, payer)
}

/// the last transaction is maximized in size, the first is not.
fn condense_right(txs: &[TransactionBuilder], payer: &Keypair) -> Result<Vec<TransactionBuilder>> {
    Ok(condense_left(&txs.to_vec().deep_reverse(), payer)?.deep_reverse())
}

/// the first transaction is maximized in size, the last is not.
fn condense_left(txs: &[TransactionBuilder], payer: &Keypair) -> Result<Vec<TransactionBuilder>> {
    let hash = Hash::new(&[0; HASH_BYTES]);
    let mut shrink_me = txs.to_vec();
    let mut condensed = vec![];
    loop {
        if shrink_me.is_empty() {
            return Ok(condensed);
        }
        let next = find_first_condensed(&shrink_me, hash, payer)?;
        condensed.push(shrink_me[0..next].ijoin());
        shrink_me = shrink_me[next..shrink_me.len()].to_vec();
    }
}

/// Searches efficiently for the largest continuous group of TransactionBuilders
/// starting from index 0 that can be merged into a single transaction without
/// exceeding the transaction size limit.
fn find_first_condensed(txs: &[TransactionBuilder], hash: Hash, payer: &Keypair) -> Result<usize> {
    let mut try_len = txs.len();
    let mut bounds = (min(txs.len(), 1), try_len);
    loop {
        if bounds.1 == bounds.0 {
            return Ok(bounds.0);
        }
        let size = txs[0..try_len].ijoin().encode(hash, payer)?.len();
        if size > MAX_TX_SIZE {
            bounds = (bounds.0, try_len - 1);
        } else {
            bounds = (try_len, bounds.1);
        }
        let ratio = MAX_TX_SIZE as f64 / size as f64;
        let mut maybe_try = (ratio * try_len as f64).round() as usize;
        maybe_try = min(bounds.1, max(bounds.0, maybe_try));
        if maybe_try == try_len {
            // if the approximated search leads to an infinite loop, fall back to a binary search.
            try_len = ((bounds.0 + bounds.1) as f64 / 2.0).round() as usize;
        } else {
            try_len = maybe_try;
        }
    }
}

fn create_transaction(
    signers: &[&Keypair],
    instructions: &[Instruction],
    blockhash: Hash,
    payer: &Keypair,
) -> Result<Transaction> {
    let mut all_signers = vec![payer];
    all_signers.extend(signers);

    Ok(Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &all_signers,
        blockhash,
    ))
}

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
        condense(&transactions, self.payer())?
            .into_iter()
            .map_async(|tx| self.send_and_confirm(tx))
            .await
    }

    async fn send_and_confirm_condensed_in_order(
        &self,
        transactions: Vec<TransactionBuilder>,
    ) -> Result<Vec<Signature>> {
        condense(&transactions, self.payer())?
            .into_iter()
            .map_async_chunked(1, |tx| self.send_and_confirm(tx))
            .await
    }
}

/// Analogous to SendTransactionBuilder, but allows you to call it with the
/// TransactionBuilder as the receiver when it would enable a cleaner
/// method-chaining syntax.
impl TransactionBuilder {
    /// SendTransactionBuilder::compile
    pub async fn compile<C: SendTransactionBuilder>(self, client: &C) -> Result<Transaction> {
        client.compile(self).await
    }

    /// SendTransactionBuilder::send_and_confirm
    pub async fn send_and_confirm<C: SendTransactionBuilder>(
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

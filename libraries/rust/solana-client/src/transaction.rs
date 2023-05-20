use anchor_lang::prelude::Pubkey;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::hash::Hash;
use solana_sdk::message::{v0, CompileError, Message, VersionedMessage};
use solana_sdk::signer::SignerError;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::cmp::{max, min};
use thiserror::Error;

use crate::util::{
    data::{Concat, DeepReverse, Join},
    keypair::clone_vec,
};

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
    /// convert transaction to a base64 string similar to one that would be
    /// submitted to rpc node. It uses fake signatures so it's not the real
    /// transaction, but it should have the same size.
    pub fn fake_encode(&self, payer: &Pubkey) -> Result<String, bincode::Error> {
        let mut compiled = Transaction::new_unsigned(Message::new(&self.instructions, Some(payer)));
        compiled.signatures.extend(
            (0..compiled.message.header.num_required_signatures as usize)
                .map(|_| Signature::new_unique()),
        );

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

/// Combines all the instructions within each of the TransactionBuilders into
/// the smallest possible number of TransactionBuilders that don't violate the
/// rules:
/// - instructions that were already grouped in a TransactionBuilder must end up
///   in the same TransactionBuilder
/// - transaction may not exceed size limit
/// - instructions order is not modified
///
/// Prioritizes bundling as much as possible with the final transaction, which
/// we're guessing will benefit more from bundling than the starting
/// transactions.
///
/// This guess comes from the fact that often you have a lot of state refresh
/// instructions that come before a final user action. Ideally all the refreshes
/// go in the same transaction with the user action. Once any get separated from
/// the user action, it doesn't really matter how they are grouped any more. But
/// you still want as many as possible with the user action.
pub fn condense(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
) -> Result<Vec<TransactionBuilder>, bincode::Error> {
    condense_right(txs, payer)
}

/// Use this when you don't care how transactions bundled, and just want all the
/// transactions delivered as fast as possible in the smallest number of
/// transactions.
pub fn condense_fast(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
) -> Result<Vec<TransactionBuilder>, bincode::Error> {
    condense_left(txs, payer)
}

/// The last transaction is maximized in size, the first is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the final transaction than those in the first transaction.
pub fn condense_right(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
) -> Result<Vec<TransactionBuilder>, bincode::Error> {
    Ok(condense_left(&txs.to_vec().deep_reverse(), payer)?.deep_reverse())
}

/// The first transaction is maximized in size, the last is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the first transaction than those in the final transaction.
pub fn condense_left(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
) -> Result<Vec<TransactionBuilder>, bincode::Error> {
    let mut shrink_me = txs.to_vec();
    let mut condensed = vec![];
    loop {
        if shrink_me.is_empty() {
            return Ok(condensed);
        }
        let next_idx = find_first_condensed(&shrink_me, payer)?;
        let next_tx = shrink_me[0..next_idx].ijoin();
        if !next_tx.instructions.is_empty() {
            condensed.push(shrink_me[0..next_idx].ijoin());
        }
        shrink_me = shrink_me[next_idx..shrink_me.len()].to_vec();
    }
}

/// Compile the instructions into a versioned transaction
pub fn compile_versioned_transaction(
    instructions: &[Instruction],
    payer: &Pubkey,
    recent_blockhash: Hash,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<VersionedTransaction, TransactionBuildError> {
    let message = VersionedMessage::V0(v0::Message::try_compile(
        payer,
        instructions,
        lookup_tables,
        recent_blockhash,
    )?);
    let tx = VersionedTransaction {
        signatures: vec![],
        message,
    };
    Ok(tx)
}

/// Searches efficiently for the largest continuous group of TransactionBuilders
/// starting from index 0 that can be merged into a single transaction without
/// exceeding the transaction size limit.
///
/// TODO: this could be modified to search from the end instead of the
/// beginning, so it would serve condense_right instead of condense_left. Then
/// condense and condense_fast could be consolidated.
fn find_first_condensed(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
) -> Result<usize, bincode::Error> {
    let mut try_len = txs.len();
    let mut bounds = (min(txs.len(), 1), try_len);
    loop {
        if bounds.1 == bounds.0 {
            return Ok(bounds.0);
        }
        let size = txs[0..try_len].ijoin().fake_encode(payer)?.len();
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

/// A type convertible to a solana transaction
pub trait ToTransaction {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction;
}

impl ToTransaction for Instruction {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = Transaction::new_unsigned(Message::new(&[self.clone()], Some(payer)));
        tx.message.recent_blockhash = recent_blockhash;

        tx.into()
    }
}

impl ToTransaction for [Instruction] {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = Transaction::new_unsigned(Message::new(self, Some(payer)));
        tx.message.recent_blockhash = recent_blockhash;

        tx.into()
    }
}

impl ToTransaction for Vec<Instruction> {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = Transaction::new_unsigned(Message::new(self, Some(payer)));
        tx.message.recent_blockhash = recent_blockhash;

        tx.into()
    }
}

impl ToTransaction for TransactionBuilder {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = Transaction::new_unsigned(Message::new(&self.instructions, Some(payer)));
        tx.partial_sign(&self.signers.iter().collect::<Vec<_>>(), recent_blockhash);

        tx.into()
    }
}

impl ToTransaction for Transaction {
    fn to_transaction(&self, _payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = self.clone();
        tx.message.recent_blockhash = recent_blockhash;

        tx.into()
    }
}

impl ToTransaction for VersionedTransaction {
    fn to_transaction(&self, _payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        let mut tx = self.clone();
        tx.message.set_recent_blockhash(recent_blockhash);

        tx
    }
}

impl<T: ToTransaction> ToTransaction for &T {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        (*self).to_transaction(payer, recent_blockhash)
    }
}

#[macro_export]
macro_rules! transactions {
    ($($item:expr),*$(,)?) => {{
        use jet_solana_client::transaction::TransactionBuilder;
        use jet_solana_client::transaction::ToTransactionBuilderVec;
        let x: Vec<TransactionBuilder> = $crate::cat![$(
            $item.to_tx_builder_vec(),
        )*];
        x
    }};
}

pub trait ToTransactionBuilderVec {
    fn to_tx_builder_vec(self) -> Vec<TransactionBuilder>;
}

impl ToTransactionBuilderVec for Instruction {
    fn to_tx_builder_vec(self) -> Vec<TransactionBuilder> {
        vec![self.into()]
    }
}
impl ToTransactionBuilderVec for Vec<Instruction> {
    fn to_tx_builder_vec(self) -> Vec<TransactionBuilder> {
        self.into_iter().map(|ix| ix.into()).collect()
    }
}
impl ToTransactionBuilderVec for TransactionBuilder {
    fn to_tx_builder_vec(self) -> Vec<TransactionBuilder> {
        vec![self]
    }
}
impl ToTransactionBuilderVec for Vec<TransactionBuilder> {
    fn to_tx_builder_vec(self) -> Vec<TransactionBuilder> {
        self
    }
}

#[derive(Error, Debug)]
pub enum TransactionBuildError {
    #[error("Error compiling versioned transaction")]
    CompileError(#[from] CompileError),
    #[error("Error signing transaction")]
    SigningError(#[from] SignerError),
}

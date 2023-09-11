use std::cmp::{max, min};
use std::collections::HashSet;

use thiserror::Error;

use anchor_lang::prelude::Pubkey;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::hash::Hash;
use solana_sdk::message::{v0, CompileError, Message, VersionedMessage};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{Signer, SignerError};
use solana_sdk::signers::Signers;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{instruction::Instruction, signature::Signature, transaction::Transaction};

use crate::lookup_tables::{exclude_useless_lookup_tables, optimize_lookup_tables};
use crate::signature::NeedsSignature;
use crate::util::data::{Concat, DeepReverse, Join};
use crate::util::keypair::clone_vec;
use crate::util::keypair::{KeypairExt, ToKeypair};

/// A group of instructions that are expected to execute in the same
/// transaction. Can be merged with other TransactionBuilder instances:
/// ```rust ignore
/// let builder = cat![builder1, builder2, builder3];
/// let builder = builder_vec.ijoin();
/// let builder = builder1.concat(builder2);
/// let builder_vec = condense(builder_vec);
/// ```
#[derive(Debug, Default)]
pub struct TransactionBuilder {
    /// see above
    pub instructions: Vec<Instruction>,
    /// Generated keypairs that will be used for the for the included
    /// instructions. Typically, this is used when an account needs to be
    /// initialized for this instruction.
    ///
    /// This usually does not include the payer or the user's wallet. Additional
    /// signatures should be provided by the application when needed. However,
    /// sometimes it may be convenient (e.g. in tests) to actually add the
    /// user's wallet into this struct before converting it to a transaction.
    pub signers: Vec<Keypair>,
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
            signers: self.signers.iter().map(|k| k.clone()).collect(),
        }
    }
}

impl From<Vec<Instruction>> for TransactionBuilder {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            signers: vec![],
        }
    }
}

/// Returns an iterator of references to all the instructions contained within
/// all the transactions. This is efficient when you just need to read the
/// instructions without owning them.
///
/// If you need owned instructions, do not use this function, unless you also
/// need to keep ownership over the transactions. It would typically be more
/// efficient to consume a vec of transaction builders with into_iter to get
/// owned instructions, rather than copying these references.
pub fn instructions(transactions: &[TransactionBuilder]) -> impl Iterator<Item = &Instruction> {
    transactions.iter().flat_map(|t| t.instructions.iter())
}

impl From<Instruction> for TransactionBuilder {
    fn from(ix: Instruction) -> Self {
        Self {
            instructions: vec![ix],
            signers: vec![],
        }
    }
}

impl TransactionBuilder {
    /// Cleans up any duplicate or unneeded signers.
    pub fn prune(&mut self) {
        let mut signer_pubkeys = HashSet::new();
        for signer in std::mem::take(&mut self.signers) {
            let pubkey = signer.pubkey();
            if !signer_pubkeys.contains(&pubkey) && self.instructions.needs_signature(pubkey) {
                signer_pubkeys.insert(pubkey);
                self.signers.push(signer);
            }
        }
    }

    /// Convert the TransactionBuilder into a solana Transaction.
    ///
    /// Handles the typical situation where the payer is the only additional
    /// signer needed. For arbitrary additional signers, use compile_custom or
    /// compile_partial.
    ///
    /// Returns error if any required signers are not provided.
    pub fn compile<S: Signer>(
        self,
        payer: &S,
        recent_blockhash: Hash,
    ) -> Result<Transaction, SignerError> {
        self.compile_custom(Some(&payer.pubkey()), &[payer], recent_blockhash)
    }

    /// Convert the TransactionBuilder into a solana Transaction.
    ///
    /// Returns error if any required signers are not provided.
    pub fn compile_custom<S: Signers>(
        self,
        payer: Option<&Pubkey>,
        signers: &S,
        recent_blockhash: Hash,
    ) -> Result<Transaction, SignerError> {
        let mut tx = self.compile_partial(payer, recent_blockhash);
        tx.try_sign(signers, recent_blockhash)?;
        Ok(tx)
    }

    /// Like compile, except that it will not fail if signers are missing.
    /// Intended to have other signatures, such as the payer's, added later.
    pub fn compile_partial(
        mut self,
        payer: Option<&Pubkey>,
        recent_blockhash: Hash,
    ) -> Transaction {
        self.prune();
        let mut tx = Transaction::new_unsigned(Message::new(&self.instructions, payer));
        tx.partial_sign(&self.signers.iter().collect::<Vec<_>>(), recent_blockhash);
        tx
    }

    /// Convert the TransactionBuilder into a VersionedTransaction using the
    /// provided lookup tables.
    ///
    /// Handles the typical situation where the payer is the only additional
    /// signer needed. For arbitrary additional signers, use compile_custom or
    /// compile_partial.
    ///
    /// Returns error if any required signers are not provided.
    ///
    /// Feel free to provide any lookup tables that you think might be useful.
    /// Only the optimal subset will be included in the transaction.
    pub fn compile_with_lookup<'a>(
        self,
        payer: &'a (impl Signer + 'a),
        lookup_tables: &[AddressLookupTableAccount],
        recent_blockhash: Hash,
    ) -> Result<VersionedTransaction, TransactionBuildError> {
        self.compile_custom_with_lookup(&payer.pubkey(), [payer], lookup_tables, recent_blockhash)
    }

    /// Convert the TransactionBuilder into a VersionedTransaction using the
    /// provided lookup tables.
    ///
    /// Returns error if any required signers are not provided.
    ///
    /// Feel free to provide any lookup tables that you think might be useful.
    /// Only the optimal subset will be included in the transaction.
    pub fn compile_custom_with_lookup<'a>(
        self,
        payer: &Pubkey,
        signers: impl IntoIterator<Item = &'a (impl Signer + 'a)>,
        lookup_tables: &[AddressLookupTableAccount],
        recent_blockhash: Hash,
    ) -> Result<VersionedTransaction, TransactionBuildError> {
        let mut tx = self.compile_partial_with_lookup(payer, lookup_tables, recent_blockhash)?;
        sign_transaction(signers, &mut tx)?;
        verify_signatures(&tx)?;
        Ok(tx)
    }

    /// Like compile, except that it will not fail if signers are missing.
    /// Intended to have other signatures, such as the payer's, added later.
    ///
    /// Feel free to provide any lookup tables that you think might be useful.
    /// Only the optimal subset will be included in the transaction.
    pub fn compile_partial_with_lookup(
        mut self,
        payer: &Pubkey,
        lookup_tables: &[AddressLookupTableAccount],
        recent_blockhash: Hash,
    ) -> Result<VersionedTransaction, TransactionBuildError> {
        self.prune();
        let optimal_tables = optimize_lookup_tables(&self.instructions, lookup_tables);
        let mut tx = create_unsigned_transaction(
            &self.instructions,
            payer,
            &optimal_tables,
            recent_blockhash,
        )?;
        sign_transaction(&self.signers, &mut tx)?;
        Ok(tx)
    }

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

    /// convert transaction to a base64 string similar to one that would be
    /// submitted to rpc node. It uses fake signatures so it's not the real
    /// transaction, but it should have the same size.
    ///
    /// Feel free to provide any lookup tables that you think might be useful.
    /// Only the optimal subset will be included in the transaction.
    pub fn fake_encode_with_lookup(
        &self,
        payer: &Pubkey,
        lookup_tables: &[AddressLookupTableAccount],
    ) -> Result<String, FakeEncodeError> {
        let optimal_tables = optimize_lookup_tables(&self.instructions, lookup_tables);
        let message = VersionedMessage::V0(v0::Message::try_compile(
            payer,
            &self.instructions,
            &optimal_tables,
            Hash::default(),
        )?);
        let tx = VersionedTransaction {
            signatures: (0..message.header().num_required_signatures as usize)
                .map(|_| Signature::new_unique())
                .collect(),
            message,
        };
        let serialized = bincode::serialize(&tx)?;
        Ok(base64::encode(serialized))
    }
}

#[derive(Error, Debug)]
pub enum FakeEncodeError {
    #[error("fake serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("fake compilation error: {0}")]
    Compilation(#[from] CompileError),
}

impl Concat for TransactionBuilder {
    fn cat(mut self, other: Self) -> Self {
        self.instructions.extend(other.instructions);
        self.signers.extend(other.signers);

        Self { ..self }
    }

    fn cat_ref(mut self, other: &Self) -> Self {
        self.instructions
            .extend(other.instructions.clone());
        self.signers.extend(other.signers.iter().map(|k| k.clone()));

        Self { ..self }
    }
}

/// Convert types to a TransactionBuilder while including signers. Serves a
/// similar purpose to From<Instruction>, but it's used when you also need to
/// add signers.
pub trait WithSigner: Sized {
    type Output;

    /// convert to a TransactionBuilder that includes this signer
    fn with_signer<K: ToKeypair>(self, signer: K) -> Self::Output {
        self.with_signers([signer])
    }

    fn without_signer(self) -> Self::Output {
        self.with_signers(Vec::<Keypair>::new())
    }

    /// convert to a TransactionBuilder<PreferredSigner> that includes these signers
    /// //todo slice
    fn with_signers<K: ToKeypair>(self, signers: impl IntoIterator<Item = K>) -> Self::Output;
}

impl WithSigner for Instruction {
    type Output = TransactionBuilder;

    fn with_signers<K: ToKeypair>(self, signers: impl IntoIterator<Item = K>) -> Self::Output {
        [self].with_signers(signers)
    }
}

impl WithSigner for &[Instruction] {
    type Output = TransactionBuilder;

    fn with_signers<K: ToKeypair>(self, signers: impl IntoIterator<Item = K>) -> Self::Output {
        TransactionBuilder {
            instructions: self.to_vec(),
            signers: clone_vec(signers),
        }
    }
}

impl WithSigner for TransactionBuilder {
    type Output = TransactionBuilder;

    fn with_signers<K: ToKeypair>(mut self, signers: impl IntoIterator<Item = K>) -> Self::Output {
        self.signers.extend(clone_vec(signers));
        TransactionBuilder {
            instructions: self.instructions,
            signers: self.signers,
        }
    }
}

impl WithSigner for Vec<TransactionBuilder> {
    type Output = Vec<TransactionBuilder>;

    fn with_signers<K: ToKeypair>(self, signers: impl IntoIterator<Item = K>) -> Self::Output {
        let signers = signers
            .into_iter()
            .map(ToKeypair::to_keypair)
            .collect::<Vec<_>>();
        self.into_iter()
            .map(|tx| tx.with_signers(clone_vec(&signers)))
            .collect()
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
///
/// TODO: lookup tables
pub fn condense(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<TransactionBuilder>, FakeEncodeError> {
    condense_right(txs, payer, lookup_tables)
}

/// Use this when you don't care how transactions bundled, and just want all the
/// transactions delivered as fast as possible in the smallest number of
/// transactions.
pub fn condense_fast(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<TransactionBuilder>, FakeEncodeError> {
    condense_left(txs, payer, lookup_tables)
}

/// The last transaction is maximized in size, the first is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the final transaction than those in the first transaction.
pub fn condense_right(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<TransactionBuilder>, FakeEncodeError> {
    Ok(condense_left(&txs.to_vec().deep_reverse(), payer, lookup_tables)?.deep_reverse())
}

/// The first transaction is maximized in size, the last is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the first transaction than those in the final transaction.
pub fn condense_left(
    txs: &[TransactionBuilder],
    payer: &Pubkey,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<TransactionBuilder>, FakeEncodeError> {
    let useful_tables = exclude_useless_lookup_tables(instructions(txs), lookup_tables);
    let mut shrink_me = txs.to_vec();
    let mut condensed = vec![];
    loop {
        if shrink_me.is_empty() {
            return Ok(condensed);
        }
        let next_idx = find_first_condensed(&shrink_me, payer, &useful_tables)?;
        let next_tx = shrink_me[0..next_idx].ijoin();
        if !next_tx.instructions.is_empty() {
            condensed.push(shrink_me[0..next_idx].ijoin());
        }
        shrink_me = shrink_me[next_idx..shrink_me.len()].to_vec();
    }
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
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<usize, FakeEncodeError> {
    let mut try_len = txs.len();
    let mut bounds = (min(txs.len(), 1), try_len);
    loop {
        if bounds.1 == bounds.0 {
            return Ok(bounds.0);
        }
        let size = if lookup_tables.is_empty() {
            txs[0..try_len].ijoin().fake_encode(payer)?.len()
        } else {
            txs[0..try_len]
                .ijoin()
                .fake_encode_with_lookup(payer, lookup_tables)?
                .len()
        };
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

/// Compile the instructions into a versioned transaction
pub fn create_unsigned_transaction(
    instructions: &[Instruction],
    payer: &Pubkey,
    lookup_tables: &[AddressLookupTableAccount],
    recent_blockhash: Hash,
) -> Result<VersionedTransaction, TransactionBuildError> {
    log::trace!("input lookup tables: {lookup_tables:?}");

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

    if let Some(lookups) = tx.message.address_table_lookups() {
        log::trace!("resolved address lookups: {lookups:?}");
    }

    log::trace!("static keys: {:?}", tx.message.static_account_keys());

    Ok(tx)
}

/// Sign a versioned transaction with keypairs
pub fn sign_transaction<'a>(
    signers: impl IntoIterator<Item = &'a (impl Signer + 'a)>,
    tx: &mut VersionedTransaction,
) -> Result<(), TransactionBuildError> {
    let to_sign = tx.message.serialize();
    tx.signatures.resize(
        tx.message.header().num_required_signatures.into(),
        Default::default(),
    );

    for signer in signers {
        let index = tx
            .message
            .static_account_keys()
            .iter()
            .position(|key| *key == signer.pubkey())
            .ok_or(SignerError::KeypairPubkeyMismatch)?;
        let signature = signer.sign_message(&to_sign);
        tx.signatures[index] = signature;
    }

    Ok(())
}

/// if there are any required signers that have not signed, returns an error
/// with a detailed message explaining the problem.
pub fn verify_signatures(tx: &VersionedTransaction) -> Result<(), SignerError> {
    use std::fmt::Write;
    let mut error_message = String::new();

    // check total
    let not_enough = tx.signatures.len() < tx.message.header().num_required_signatures.into();
    if not_enough {
        write!(
            error_message,
            "Not enough signatures. expected {} but got {}. ",
            tx.message.header().num_required_signatures,
            tx.signatures.len()
        )
        .expect("string formatting should never fail");
    }

    // check each
    let mut fail_pubkeys = vec![];
    let keys = tx.message.static_account_keys();
    for (index, verified) in tx.verify_with_results().into_iter().enumerate() {
        if !verified {
            fail_pubkeys.push((index, keys.get(index)));
        }
    }
    if !fail_pubkeys.is_empty() {
        write!(
            error_message,
            "Signatures failed verification for unknown reasons: {fail_pubkeys:#?}"
        )
        .expect("string formatting should never fail");
    }

    // aggregate checks
    if !not_enough && fail_pubkeys.is_empty() && error_message.is_empty() {
        Ok(())
    } else {
        Err(SignerError::Custom(error_message))
    }
}

/// Compile and sign the instructions into a versioned transaction
pub fn create_signed_transaction(
    instructions: &[Instruction],
    signer: &Keypair,
    lookup_tables: &[AddressLookupTableAccount],
    recent_blockhash: Hash,
) -> Result<VersionedTransaction, TransactionBuildError> {
    let mut tx = create_unsigned_transaction(
        instructions,
        &signer.pubkey(),
        lookup_tables,
        recent_blockhash,
    )?;

    sign_transaction([signer], &mut tx)?;
    Ok(tx)
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
        self.clone()
            .compile_partial(Some(payer), recent_blockhash)
            .into()
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

/// ```pseudo-code
/// fn transactions!(varargs...: impl ToTransactionBuilderVec) -> Vec<TransactionBuilder>
/// ```
/// Converts each input into a Vec<TransactionBuilder>,  
/// then concatenates the vecs into a unified Vec<TransactionBuilder>.
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

/// ```pseudo-code
/// fn tx!(varargs...: impl ToTransactionBuilderVec) -> TransactionBuilder
/// ```
/// Combines all enclosed items into a single TransactionBuilder.
#[macro_export]
macro_rules! tx {
    ($($item:expr),*$(,)?) => {{
        use jet_solana_client::transaction::TransactionBuilder;
        use jet_solana_client::transaction::ToTransactionBuilderVec;
        use jet_solana_client::util::data::Join;
        let x: TransactionBuilder = $crate::cat![$(
            $item.to_tx_builder_vec(),
        )*].ijoin();
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

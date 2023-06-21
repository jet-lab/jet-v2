use anchor_lang::prelude::Pubkey;
use solana_sdk::hash::Hash;
use solana_sdk::message::Message;
use solana_sdk::signer::{Signer, SignerError};
use solana_sdk::signers::Signers;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{instruction::Instruction, signature::Signature, transaction::Transaction};
use std::cmp::{max, min};
use std::ops::Deref;

use crate::signature::FlexSigners;
use crate::signature::StandardSigner;
use crate::util::data::{Concat, DeepReverse, Join};

pub type TransactionBuilder = InstructionBundle<StandardSigner>;

/// A group of instructions that are expected to execute in the same
/// transaction. Can be merged with other InstructionBundle instances:
/// ```rust ignore
/// let bundle = cat![bundle1, bundle2, bundle3];
/// let bundle = bundle_vec.ijoin();
/// let bundle = bundle1.concat(bundle2);
/// let bundle_vec = condense(bundle_vec);
/// ```
#[derive(Debug)]
pub struct InstructionBundle<K> {
    /// see above
    pub instructions: Vec<Instruction>,
    /// The signers that are required for the included instructions.
    /// May not include the transaction fee payer.
    pub signers: Vec<K>,
}

impl<K> Default for InstructionBundle<K> {
    fn default() -> Self {
        Self {
            instructions: vec![],
            signers: vec![],
        }
    }
}

impl<K> DeepReverse for InstructionBundle<K> {
    fn deep_reverse(mut self) -> Self {
        self.instructions.reverse();
        self
    }
}

impl<K: Clone> Clone for InstructionBundle<K> {
    fn clone(&self) -> Self {
        Self {
            instructions: self.instructions.clone(),
            signers: self.signers.clone(),
        }
    }
}

impl<K> From<Vec<Instruction>> for InstructionBundle<K> {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            signers: vec![],
        }
    }
}

impl<K> From<Instruction> for InstructionBundle<K> {
    fn from(ix: Instruction) -> Self {
        Self {
            instructions: vec![ix],
            signers: vec![],
        }
    }
}

impl<K> InstructionBundle<K> {
    /// Convert the InstructionBundle into a solana Transaction, as long as the
    /// signers implement Signer. Returns error if any required signers are not
    /// provided.
    ///
    /// Ideally, this function would be generic over either any Deref<Signer>
    /// type OR any *direct* Signer type. Unfortunately, this is not possible
    /// without specialization or negative trait bounds. Instead, you'll just
    /// need to wrap direct Signer types in something like Box to make them
    /// compatible. `InstructionBundle::signers_into` can help with this.
    pub fn compile<Payer, KSigner, PSigner>(
        self,
        payer: Payer,
        recent_blockhash: Hash,
    ) -> Result<Transaction, SignerError>
    where
        K: Deref<Target = KSigner>,
        Payer: Deref<Target = PSigner>,
        KSigner: ?Sized + Signer,
        PSigner: ?Sized + Signer,
    {
        let payer_pubkey = payer.pubkey();
        let signers = FlexSigners {
            signers: FlexSigners::from_non_signers(self.signers),
            signer_vec: vec![payer],
        };
        try_new_tx(
            &self.instructions,
            Some(&payer_pubkey),
            &signers,
            recent_blockhash,
        )
    }

    /// Like compile, except that it will not fail if signers are missing.
    /// Intended to accept the payer as a signer later.
    pub fn compile_partial<KSigner>(
        self,
        payer: Option<&Pubkey>,
        recent_blockhash: Hash,
    ) -> Transaction
    where
        K: Deref<Target = KSigner>,
        KSigner: ?Sized + Signer,
    {
        new_tx_partial(
            &self.instructions,
            payer,
            &FlexSigners::from_non_signers(self.signers),
            recent_blockhash,
        )
    }

    /// Like compile, except that it assumes the payer is already included in
    /// the signers, so only a pubkey is needed.
    pub fn compile_with_included_payer<KSigner>(
        self,
        payer: Option<&Pubkey>,
        recent_blockhash: Hash,
    ) -> Result<Transaction, SignerError>
    where
        K: Deref<Target = KSigner>,
        KSigner: ?Sized + Signer,
    {
        try_new_tx(
            &self.instructions,
            payer,
            &FlexSigners::from_non_signers(self.signers),
            recent_blockhash,
        )
    }

    pub fn signers_into<NewK: From<K>>(self) -> InstructionBundle<NewK> {
        self.map_signers(|s| NewK::from(s))
    }

    pub fn map_signers<NewK, F: FnMut(K) -> NewK>(self, f: F) -> InstructionBundle<NewK> {
        InstructionBundle {
            signers: self.signers.into_iter().map(f).collect(),
            instructions: self.instructions,
        }
    }

    /// Removes all the signers so you can convert this into any arbitrary type.
    /// Useful when converting from Pubkey to an actual signer type. You'll need
    /// to manually ensure all the required signers do eventually sign the
    /// transaction
    pub fn without_signers<NewK>(self) -> InstructionBundle<NewK> {
        InstructionBundle {
            signers: vec![],
            instructions: self.instructions,
        }
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
}

/// Like Transaction::new_signed_with_payer, except that it returns an error
/// instead of panicking when a required signer is missing.
pub fn try_new_tx<T: Signers>(
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    signers: &T,
    recent_blockhash: Hash,
) -> Result<Transaction, SignerError> {
    let mut tx = Transaction::new_unsigned(Message::new(instructions, payer));
    tx.try_sign(signers, recent_blockhash)?;
    Ok(tx)
}

/// Like Transaction::new_signed_with_payer, except that it will successfully
/// create a Transaction even if some required signers are missing.
pub fn new_tx_partial<T: Signers>(
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    signers: &T,
    recent_blockhash: Hash,
) -> Transaction {
    let mut tx = Transaction::new_unsigned(Message::new(instructions, payer));
    tx.partial_sign(signers, recent_blockhash);
    tx
}

impl<K: Clone> Concat for InstructionBundle<K> {
    fn cat(mut self, other: Self) -> Self {
        self.instructions.extend(other.instructions.into_iter());
        self.signers.extend(other.signers.into_iter());

        Self { ..self }
    }

    fn cat_ref(mut self, other: &Self) -> Self {
        self.instructions
            .extend(other.instructions.clone().into_iter());
        self.signers.extend(other.signers.clone().into_iter());

        Self { ..self }
    }
}

/// Convert types to a TransactionBuilder while including signers. Serves a
/// similar purpose to From<Instruction>, but it's used when you also need to
/// add signers.
pub trait WithSigner<K>: Sized {
    /// convert to a TransactionBuilder that includes this signer
    fn with_signer(self, signer: K) -> InstructionBundle<K> {
        self.with_signers(vec![signer])
    }
    /// convert to a InstructionBundle<PreferredSigner> that includes these signers
    fn with_signers(self, signers: Vec<K>) -> InstructionBundle<K>;
}

impl<K> WithSigner<K> for Instruction {
    fn with_signers(self, signers: Vec<K>) -> InstructionBundle<K> {
        vec![self].with_signers(signers)
    }
}

impl<K> WithSigner<K> for &[Instruction] {
    fn with_signers(self, signers: Vec<K>) -> InstructionBundle<K> {
        InstructionBundle {
            instructions: self.to_vec(),
            signers,
        }
    }
}

impl<K> WithSigner<K> for InstructionBundle<K> {
    fn with_signers(mut self, signers: Vec<K>) -> InstructionBundle<K> {
        self.signers.extend(signers.into_iter().collect::<Vec<_>>());
        InstructionBundle {
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
pub fn condense<K: Clone>(
    txs: &[InstructionBundle<K>],
    payer: &Pubkey,
) -> Result<Vec<InstructionBundle<K>>, bincode::Error> {
    condense_right(txs, payer)
}

/// Use this when you don't care how transactions bundled, and just want all the
/// transactions delivered as fast as possible in the smallest number of
/// transactions.
pub fn condense_fast<K: Clone>(
    txs: &[InstructionBundle<K>],
    payer: &Pubkey,
) -> Result<Vec<InstructionBundle<K>>, bincode::Error> {
    condense_left(txs, payer)
}

/// The last transaction is maximized in size, the first is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the final transaction than those in the first transaction.
pub fn condense_right<K: Clone>(
    txs: &[InstructionBundle<K>],
    payer: &Pubkey,
) -> Result<Vec<InstructionBundle<K>>, bincode::Error> {
    Ok(condense_left(&txs.to_vec().deep_reverse(), payer)?.deep_reverse())
}

/// The first transaction is maximized in size, the last is not.
/// - Use when it's more important to bundle as much as possible with the
///   instructions in the first transaction than those in the final transaction.
pub fn condense_left<K: Clone>(
    txs: &[InstructionBundle<K>],
    payer: &Pubkey,
) -> Result<Vec<InstructionBundle<K>>, bincode::Error> {
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

/// Searches efficiently for the largest continuous group of TransactionBuilders
/// starting from index 0 that can be merged into a single transaction without
/// exceeding the transaction size limit.
///
/// TODO: this could be modified to search from the end instead of the
/// beginning, so it would serve condense_right instead of condense_left. Then
/// condense and condense_fast could be consolidated.
fn find_first_condensed<K: Clone>(
    txs: &[InstructionBundle<K>],
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

impl<K, KSigner> ToTransaction for InstructionBundle<K>
where
    K: Deref<Target = KSigner> + Clone,
    KSigner: ?Sized + Signer,
{
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

/// The main goal here is to ensure that the *code* compiles without error. Any
/// changes to the compile function should not break its compatibility with any
/// of these types.
///
/// It's also nice to see that the *transaction* compiles without error, so this
/// runs as a test instead of just being an uncalled function.
#[test]
fn instruction_bundle_compile_is_sufficiently_generic() -> Result<(), SignerError> {
    use solana_sdk::signature::Keypair;
    use std::sync::Arc;

    let hash = Hash::default();
    let k = Keypair::new();

    InstructionBundle::<Keypair>::default()
        .signers_into::<Box<Keypair>>()
        .compile(&k, hash)?;
    InstructionBundle::<Keypair>::default()
        .signers_into::<Box<dyn Signer>>()
        .compile(&k, hash)?;
    InstructionBundle::<&Keypair>::default().compile(&k, hash)?;
    InstructionBundle::<Arc<Keypair>>::default().compile(&k, hash)?;
    InstructionBundle::<Box<dyn Signer>>::default().compile(&k, hash)?;

    let k = Box::new(Keypair::new());
    InstructionBundle::<Keypair>::default()
        .signers_into::<Arc<Keypair>>()
        .compile(&*k, hash)?;
    InstructionBundle::<Keypair>::default()
        .signers_into::<Box<dyn Signer>>()
        .compile(&*k, hash)?;
    InstructionBundle::<&Keypair>::default().compile(&*k, hash)?;
    InstructionBundle::<Arc<Keypair>>::default().compile(&*k, hash)?;
    InstructionBundle::<Box<dyn Signer>>::default().compile(&*k, hash)?;

    let k: Box<dyn Signer> = Box::new(Keypair::new());
    InstructionBundle::<Keypair>::default()
        .signers_into::<Box<Keypair>>()
        .compile(&*k, hash)?;
    InstructionBundle::<Keypair>::default()
        .signers_into::<Box<dyn Signer>>()
        .compile(&*k, hash)?;
    InstructionBundle::<&Keypair>::default().compile(&*k, hash)?;
    InstructionBundle::<Arc<Keypair>>::default().compile(&*k, hash)?;
    InstructionBundle::<Box<dyn Signer>>::default().compile(&*k, hash)?;

    Ok(())
}

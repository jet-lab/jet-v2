use solana_sdk::hash::{Hash, HASH_BYTES};
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{instruction::Instruction, signature::Keypair, transaction::Transaction};
use std::cmp::{max, min};

use crate::{
    solana::keypair::clone_vec,
    util::data::{Concat, Join},
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
    pub fn encode(&self, hash: Hash) -> Result<String, bincode::Error> {
        let compiled = create_transaction(
            &self.signers.iter().collect::<Vec<&Keypair>>(),
            &self.instructions,
            hash,
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

/// Combines all the instructions within each of the TransactionBuilders into the smallest
///  possible number of TransactionBuilders that don't violate the rules:
/// - instructions that were already grouped in a TransactionBuilder must end up in the same TransactionBuilder
/// - transaction may not exceed size limit
/// - instructions order is not modified
pub fn condense(txs: &[TransactionBuilder]) -> Result<Vec<TransactionBuilder>, bincode::Error> {
    let hash = Hash::new(&[0; HASH_BYTES]);
    let mut shrink_me = txs.to_vec();
    let mut condensed = vec![];
    loop {
        if shrink_me.is_empty() {
            return Ok(condensed);
        }
        let next = find_first_condensed(&shrink_me, hash)?;
        condensed.push(shrink_me[0..next].ijoin());
        shrink_me = shrink_me[next..shrink_me.len()].to_vec();
    }
}

/// Searches efficiently for the largest continuous group of TransactionBuilders
/// starting from index 0 that can be merged into a single transaction without
/// exceeding the transaction size limit.
fn find_first_condensed(txs: &[TransactionBuilder], hash: Hash) -> Result<usize, bincode::Error> {
    let mut try_len = txs.len();
    let mut bounds = (min(txs.len(), 1), try_len);
    loop {
        if bounds.1 == bounds.0 {
            return Ok(bounds.0);
        }
        let size = txs[0..try_len].ijoin().encode(hash)?.len() + 64;
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
) -> Transaction {
    let mut tx = Transaction::new_unsigned(Message::new(instructions, None));
    let signers = signers.to_vec();

    tx.partial_sign(&signers, blockhash);

    tx
}

/// A type convertible to a solana transaction
pub trait ToTransaction {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction;
}

impl ToTransaction for Instruction {
    fn to_transaction(&self, payer: &Pubkey, _recent_blockhash: Hash) -> VersionedTransaction {
        Transaction::new_unsigned(Message::new(&[self.clone()], Some(payer))).into()
    }
}

impl ToTransaction for [Instruction] {
    fn to_transaction(&self, payer: &Pubkey, _recent_blockhash: Hash) -> VersionedTransaction {
        Transaction::new_unsigned(Message::new(self, Some(payer))).into()
    }
}

impl ToTransaction for Vec<Instruction> {
    fn to_transaction(&self, payer: &Pubkey, _recent_blockhash: Hash) -> VersionedTransaction {
        Transaction::new_unsigned(Message::new(self, Some(payer))).into()
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
    fn to_transaction(&self, _payer: &Pubkey, _recent_blockhash: Hash) -> VersionedTransaction {
        self.clone().into()
    }
}

impl ToTransaction for VersionedTransaction {
    fn to_transaction(&self, _payer: &Pubkey, _recent_blockhash: Hash) -> VersionedTransaction {
        self.clone()
    }
}

impl<T: ToTransaction> ToTransaction for &T {
    fn to_transaction(&self, payer: &Pubkey, recent_blockhash: Hash) -> VersionedTransaction {
        (*self).to_transaction(payer, recent_blockhash)
    }
}

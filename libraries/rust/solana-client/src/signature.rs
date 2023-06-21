use std::{ops::Deref, sync::Arc};

use anchor_lang::prelude::Pubkey;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature},
    signer::{Signer, SignerError},
    signers::Signers,
    transaction::VersionedTransaction,
};

use crate::util::keypair::KeypairExt;

pub type StandardSigner = Arc<Keypair>;

pub trait NeedsSignature {
    fn needs_signature(&self, potential_signer: Pubkey) -> bool;
}

impl NeedsSignature for Instruction {
    fn needs_signature(&self, potential_signer: Pubkey) -> bool {
        self.accounts
            .iter()
            .any(|a| a.is_signer && potential_signer == a.pubkey)
    }
}

impl NeedsSignature for Vec<Instruction> {
    fn needs_signature(&self, potential_signer: Pubkey) -> bool {
        self.iter().any(|ix| ix.needs_signature(potential_signer))
    }
}

/// Account to act upon, and the signer to authorize the action.
pub struct Authorization {
    pub address: Pubkey,
    pub authority: Keypair,
}

impl Clone for Authorization {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            authority: self.authority.clone(),
        }
    }
}

/// Utility for partially signing versioned transactions directly with a keypair
pub fn sign_versioned_transaction(keypair: &Keypair, tx: &mut VersionedTransaction) {
    let signature = keypair.sign_message(tx.message.serialize().as_slice());
    let index = tx
        .message
        .static_account_keys()
        .iter()
        .position(|key| *key == keypair.pubkey())
        .expect("given transaction has no matching pubkey for the signer");

    tx.signatures.resize(
        tx.message.header().num_required_signatures.into(),
        Default::default(),
    );
    tx.signatures[index] = signature;
}

pub trait StandardizeSigner {
    /// Converts into the standard signer used in TransactionBuilder
    fn standardize(&self) -> StandardSigner;
}
pub trait StandardizeSigners {
    /// Converts collection items into the standard signer used in TransactionBuilder
    fn standardize(self) -> Vec<StandardSigner>;
}

impl StandardizeSigner for Keypair {
    fn standardize(&self) -> StandardSigner {
        Arc::new(self.clone())
    }
}

impl<'a, I: IntoIterator<Item = &'a K>, K: StandardizeSigner + 'a> StandardizeSigners for I {
    fn standardize(self) -> Vec<StandardSigner> {
        self.into_iter().map(|k| k.standardize()).collect()
    }
}

/// Provides an implementation of `Signers` for composite signer types that do
/// not already implement `Signers`. Can be used to implement `Signers` for:
/// - A collection of items that each Deref to `Signer` but the collection
///   itself is not `Signers`
/// - A combination of two different data structures (one containing `Signer`s
///   and the other implementing `Signers`)
///
/// This struct can be used recursively for arbitrary complexity of `Signer`
/// composition.
pub struct FlexSigners<T, V> {
    /// T: Signers
    pub signers: T,
    /// V = Vec<U> where U: Signer
    pub signer_vec: V,
}

impl<U, V, S> FlexSigners<Vec<&dyn Signer>, V>
where
    for<'a> &'a V: IntoIterator<Item = &'a U>,
    U: Deref<Target = S>,
    S: ?Sized + Signer,
{
    /// Use this to seed a FlexSigners when none of your signers are in a data
    /// structure that already implements Signers.
    pub fn from_non_signers(signer_vec: V) -> Self {
        Self {
            signers: vec![],
            signer_vec,
        }
    }
}

impl<T, U, V, S> Signers for FlexSigners<T, V>
where
    T: Signers,
    for<'a> &'a V: IntoIterator<Item = &'a U>,
    U: Deref<Target = S>,
    S: ?Sized + Signer,
{
    fn pubkeys(&self) -> Vec<Pubkey> {
        let mut pubs = self
            .signer_vec
            .into_iter()
            .map(|keypair| keypair.pubkey())
            .collect::<Vec<Pubkey>>();
        pubs.extend(self.signers.pubkeys());
        pubs
    }

    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError> {
        let mut pubs = Vec::new();
        for keypair in self.signer_vec.into_iter() {
            pubs.push(keypair.try_pubkey()?);
        }
        pubs.extend(self.signers.try_pubkeys()?);
        Ok(pubs)
    }

    fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
        let mut sigs: Vec<_> = self
            .signer_vec
            .into_iter()
            .map(|keypair| keypair.sign_message(message))
            .collect();
        sigs.extend(self.signers.sign_message(message));
        sigs
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Vec<Signature>, SignerError> {
        let mut sigs = Vec::new();
        for keypair in self.signer_vec.into_iter() {
            sigs.push(keypair.try_sign_message(message)?);
        }
        sigs.extend(self.signers.try_sign_message(message)?);
        Ok(sigs)
    }

    fn is_interactive(&self) -> bool {
        self.signer_vec.into_iter().any(|s| s.is_interactive()) || self.signers.is_interactive()
    }
}

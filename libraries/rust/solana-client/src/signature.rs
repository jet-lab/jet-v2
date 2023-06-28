use anchor_lang::prelude::Pubkey;
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, transaction::VersionedTransaction,
};

use crate::{transaction::TransactionBuilder, util::keypair::KeypairExt};

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

impl NeedsSignature for TransactionBuilder {
    fn needs_signature(&self, potential_signer: Pubkey) -> bool {
        self.instructions.needs_signature(potential_signer)
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

use anchor_lang::prelude::Pubkey;
use solana_sdk::{instruction::Instruction, signature::Keypair};

use crate::util::keypair::KeypairExt;

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

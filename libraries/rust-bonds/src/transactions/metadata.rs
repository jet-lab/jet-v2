use solana_sdk::{hash::Hash, signature::Keypair, signer::Signer, transaction::Transaction};

use crate::instructions::metadata::*;

pub fn authorize_crank_signer_transaction(
    crank_signer: &Keypair,
    authority: &Keypair,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = &[authorize_crank_signer_instruction(
        &crank_signer.pubkey(),
        &authority.pubkey(),
        &payer.pubkey(),
    )];

    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &[authority, payer],
        recent_blockhash,
    )
}

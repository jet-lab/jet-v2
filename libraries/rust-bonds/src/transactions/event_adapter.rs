use solana_sdk::{
    hash::Hash, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::instructions::{pop_adapter_events_instruction, register_adapter_instruction};

pub fn register_adapter_transaction(
    bond_manager_key: &Pubkey,
    user: &Keypair,
    owner: &Keypair,
    payer: &Keypair,
    num_events: u32,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let register_adapter = register_adapter_instruction(
            bond_manager_key,
            &user.pubkey(),
            &owner.pubkey(),
            &payer.pubkey(),
            num_events,
        );
        &[register_adapter]
    };
    let signing_keypairs = &[user, owner, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn pop_adapter_events_transaction(
    bonds_pid: &Pubkey,
    adapter_queue_key: &Pubkey,
    owner: &Keypair,
    payer: &Keypair,
    num_events: u32,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let pop_adapter_events = pop_adapter_events_instruction(
            bonds_pid,
            adapter_queue_key,
            &owner.pubkey(),
            num_events,
        );
        &[pop_adapter_events]
    };
    let signing_keypairs = &[owner, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

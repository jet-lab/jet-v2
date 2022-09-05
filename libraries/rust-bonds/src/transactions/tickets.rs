#![allow(clippy::too_many_arguments)]
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::instructions::*;

pub fn exchange_tokens_transaction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    underlying_token_mint: &Pubkey,
    user_authority_keypair: &Keypair,
    amount: u64,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let exchange_tokens = exchange_tokens_instruction(
            jet_bonds_pid,
            bond_manager_key,
            underlying_token_mint,
            &user_authority_keypair.pubkey(),
            amount,
        );
        &[exchange_tokens]
    };
    let signing_keypairs = &[user_authority_keypair, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn initialize_bond_ticket_account_transaction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    recipient_key: &Pubkey,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let initialize_bond_ticket_account = intitialize_bond_ticket_account_instruction(
            jet_bonds_pid,
            bond_manager_key,
            recipient_key,
            &payer.pubkey(),
        );
        &[initialize_bond_ticket_account]
    };
    let signing_keypairs = &[payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn initialize_bond_manager_transaction(
    jet_bonds_pid: &Pubkey,
    underlying_token_mint_key: &Pubkey,
    program_authority: &Keypair,
    oracle: Option<Pubkey>,
    version_tag: u64,
    duration: i64,
    conversion_factor: i8,
    seed: u64,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let initialize_bond_manager = initialize_bond_manager_instruction(
            jet_bonds_pid,
            underlying_token_mint_key,
            &program_authority.pubkey(),
            &payer.pubkey(),
            oracle,
            version_tag,
            duration,
            conversion_factor,
            seed,
        );
        &[initialize_bond_manager]
    };
    let signing_keypairs = &[program_authority, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
pub fn redeem_ticket_transaction(
    jet_bonds_pid: &Pubkey,
    ticket_key: &Pubkey,
    ticket_holder: &Keypair,
    claimant_token_account_key: &Pubkey,
    underlying_token_vault_key: &Pubkey,
    bond_manager_key: &Pubkey,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let redeem_claim = redeem_ticket_instruction(
            jet_bonds_pid,
            ticket_key,
            &ticket_holder.pubkey(),
            claimant_token_account_key,
            underlying_token_vault_key,
            bond_manager_key,
        );
        &[redeem_claim]
    };
    let signing_keypairs = &[ticket_holder, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
pub fn stake_bond_tickets_transaction(
    bond_manager_key: &Pubkey,
    ticket_holder: &Keypair,
    ticket_seed: Vec<u8>,
    amount: u64,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let stake_bond_tickets = stake_bond_tickets_instruction(
            bond_manager_key,
            &ticket_holder.pubkey(),
            &payer.pubkey(),
            ticket_seed,
            amount,
        );
        &[stake_bond_tickets]
    };
    let signing_keypairs = &[ticket_holder, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

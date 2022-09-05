#![allow(clippy::too_many_arguments)]

use jet_bonds::orderbook::state::{AssetKind, OrderParams, OrderSide};
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::instructions::*;

pub fn initialize_event_queue_transaction(
    jet_bonds_pid: &Pubkey,
    event_queue: &Keypair,
    payer: &Keypair,
    rent: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let initialize_event_queue = initialize_event_queue_instruction(
            jet_bonds_pid,
            &event_queue.pubkey(),
            &payer.pubkey(),
            rent,
        );
        &[initialize_event_queue]
    };
    let signing_keypairs = &[event_queue, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
pub fn initialize_orderbook_slab_transaction(
    jet_bonds_pid: &Pubkey,
    slab: &Keypair,
    payer: &Keypair,
    rent: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let initialize_orderbook_slab = initialize_orderbook_slab_instruction(
            jet_bonds_pid,
            &slab.pubkey(),
            &payer.pubkey(),
            rent,
        );
        &[initialize_orderbook_slab]
    };
    let signing_keypairs = &[slab, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
pub fn initialize_orderbook_transaction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    program_authority: &Keypair,
    payer: &Keypair,
    min_base_order_size: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let initialize_orderbook = initialize_orderbook_instruction(
            jet_bonds_pid,
            bond_manager_key,
            event_queue_key,
            bids_key,
            asks_key,
            &payer.pubkey(),
            &program_authority.pubkey(),
            min_base_order_size,
        );
        &[initialize_orderbook]
    };
    let signing_keypairs = &[program_authority, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn initialize_orderbook_user_transaction(
    bond_manager_key: &Pubkey,
    user: &Keypair,
    payer: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let deposit = initialize_orderbook_user_instruction(
            &user.pubkey(),
            bond_manager_key,
            &payer.pubkey(),
        );
        &[deposit]
    };
    let signing_keypairs = &[user, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
pub fn deposit_transaction(
    bond_manager_key: &Pubkey,
    underlying_token_mint: &Pubkey,
    orderbook_user_account_key: &Pubkey,
    user_token_vault_authority: &Keypair,
    payer: &Keypair,
    amount: u64,
    kind: AssetKind,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let deposit_instruction = deposit_instruction(
            bond_manager_key,
            underlying_token_mint,
            orderbook_user_account_key,
            &user_token_vault_authority.pubkey(),
            amount,
            kind,
        );
        &[deposit_instruction]
    };
    let signing_keypairs = &[user_token_vault_authority, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn place_order_transaction(
    bond_manager_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    adapter: Option<&Pubkey>,
    user_keypair: &Keypair,
    payer: &Keypair,
    order_side: OrderSide,
    order_params: OrderParams,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let place_order = place_order_instruction(
            bond_manager_key,
            &user_keypair.pubkey(),
            event_queue_key,
            bids_key,
            asks_key,
            adapter,
            order_side,
            order_params,
        );
        &[place_order]
    };
    let signing_keypairs = &[user_keypair, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn place_order_authorized_transaction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    user: &Keypair,
    payer: &Keypair,
    order_side: OrderSide,
    order_params: OrderParams,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let place_order_authorized = place_order_authorized_instruction(
            *jet_bonds_pid,
            *bond_manager_key,
            *event_queue_key,
            *bids_key,
            *asks_key,
            user.pubkey(),
            order_side,
            order_params,
        );
        &[place_order_authorized]
    };
    let signing_keypairs = &[payer, user];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn cancel_order_transaction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    orderbook_market_state_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    orderbook_user_account_key: &Pubkey,
    user_keypair: &Keypair,
    payer: &Keypair,
    order_id: u128,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let cancel_order = cancel_order_instruction(
            jet_bonds_pid,
            bond_manager_key,
            &user_keypair.pubkey(),
            orderbook_market_state_key,
            event_queue_key,
            bids_key,
            asks_key,
            orderbook_user_account_key,
            order_id,
        );
        &[cancel_order]
    };
    let signing_keypairs = &[user_keypair, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub fn consume_events_transaction(
    bond_manager_key: &Pubkey,
    orderbook_market_state_key: &Pubkey,
    event_queue_key: &Pubkey,
    crank_metadata_key: &Pubkey,
    crank_signer: &Keypair,
    remaining_accounts: &[&Pubkey],
    payer: &Keypair,
    num_events: usize,
    seed_bytes: Vec<Vec<u8>>,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let consume_events = consume_events_instruction(
            bond_manager_key,
            orderbook_market_state_key,
            event_queue_key,
            crank_metadata_key,
            &crank_signer.pubkey(),
            &payer.pubkey(),
            remaining_accounts,
            num_events,
            seed_bytes,
        );
        &[consume_events]
    };
    let signing_keypairs = &[crank_signer, payer];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

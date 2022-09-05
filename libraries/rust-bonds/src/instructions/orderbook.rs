#![allow(clippy::too_many_arguments)]
use anchor_lang::{prelude::Rent, InstructionData, ToAccountMetas};
use jet_bonds::{
    control::instructions::InitializeOrderbookParams,
    orderbook::state::{AssetKind, OrderParams, OrderSide, EVENT_QUEUE_LEN, ORDERBOOK_SLAB_LEN},
    seeds,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction, system_program,
    sysvar::SysvarId,
};
use spl_associated_token_account::get_associated_token_address;

use crate::pda;

pub fn initialize_event_queue_instruction(
    jet_bonds_pid: &Pubkey,
    event_queue_key: &Pubkey,
    payer_key: &Pubkey,
    rent: u64,
) -> Instruction {
    system_instruction::create_account(
        payer_key,
        event_queue_key,
        rent,
        EVENT_QUEUE_LEN as u64,
        jet_bonds_pid,
    )
}
pub fn initialize_orderbook_slab_instruction(
    jet_bonds_pid: &Pubkey,
    slab_key: &Pubkey,
    payer_key: &Pubkey,
    rent: u64,
) -> Instruction {
    system_instruction::create_account(
        payer_key,
        slab_key,
        rent,
        ORDERBOOK_SLAB_LEN as u64,
        jet_bonds_pid,
    )
}
pub fn initialize_orderbook_instruction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    payer_key: &Pubkey,
    program_authority_key: &Pubkey,
    min_base_order_size: u64,
) -> Instruction {
    let (orderbook_market_state_key, _) = Pubkey::find_program_address(
        &[b"orderbook_market_state", bond_manager_key.as_ref()],
        jet_bonds_pid,
    );

    let accounts = vec![
        AccountMeta::new(*bond_manager_key, false),
        AccountMeta::new(orderbook_market_state_key, false),
        AccountMeta::new(*event_queue_key, false),
        AccountMeta::new(*bids_key, false),
        AccountMeta::new(*asks_key, false),
        AccountMeta::new_readonly(*program_authority_key, true),
        AccountMeta::new(*payer_key, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let data = {
        let params = InitializeOrderbookParams {
            min_base_order_size,
        };
        jet_bonds::instruction::InitializeOrderbook { params }.data()
    };

    Instruction {
        program_id: *jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn initialize_orderbook_user_instruction(
    user_key: &Pubkey,
    bond_manager_key: &Pubkey,
    payer_key: &Pubkey,
) -> Instruction {
    let orderbook_user_account = pda(&[
        b"orderbook_user",
        bond_manager_key.as_ref(),
        user_key.as_ref(),
    ]);

    let accounts = jet_bonds::accounts::InitializeOrderbookUser {
        orderbook_user_account,
        user: *user_key,
        bond_manager: *bond_manager_key,
        claims: pda(&[seeds::CLAIM_NOTES, orderbook_user_account.as_ref()]),
        claims_mint: pda(&[seeds::CLAIM_NOTES, bond_manager_key.as_ref()]),
        payer: *payer_key,
        rent: Rent::id(),
        token_program: spl_token::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let data = jet_bonds::instruction::InitializeOrderbookUser {}.data();
    Instruction {
        program_id: jet_bonds::ID,
        accounts,
        data,
    }
}

pub fn deposit_instruction(
    bond_manager_key: &Pubkey,
    underlying_token_mint: &Pubkey,
    orderbook_user_account_key: &Pubkey,
    owner_wallet: &Pubkey,
    amount: u64,
    kind: AssetKind,
) -> Instruction {
    let underlying_token_vault_key = Pubkey::find_program_address(
        &[
            b"underlying_token_vault".as_ref(),
            bond_manager_key.as_ref(),
        ],
        &jet_bonds::ID,
    )
    .0;
    let bond_ticket_mint_key = Pubkey::find_program_address(
        &[b"bond_ticket_mint".as_ref(), bond_manager_key.as_ref()],
        &jet_bonds::ID,
    )
    .0;
    let mint = match kind {
        AssetKind::BondTicket => &bond_ticket_mint_key,
        AssetKind::UnderlyingToken => underlying_token_mint,
    };
    let user_token_vault = get_associated_token_address(owner_wallet, mint);
    let accounts = jet_bonds::accounts::Deposit {
        orderbook_user_account: *orderbook_user_account_key,
        bond_manager: *bond_manager_key,
        user_token_vault,
        user_token_vault_authority: *owner_wallet,
        underlying_token_vault: underlying_token_vault_key,
        bond_ticket_mint: bond_ticket_mint_key,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);
    let data = jet_bonds::instruction::Deposit { amount, kind }.data();
    Instruction {
        program_id: jet_bonds::ID,
        accounts,
        data,
    }
}

pub fn withdraw_instruction(
    jet_bonds_pid: Pubkey,
    bond_manager_key: Pubkey,
    orderbook_user_account_key: Pubkey,
    user_token_vault_key: Pubkey,
    market_token_vault_key: Pubkey,
    amount: u64,
    kind: AssetKind,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(bond_manager_key, false),
        AccountMeta::new(orderbook_user_account_key, false),
        AccountMeta::new(user_token_vault_key, false),
        AccountMeta::new(market_token_vault_key, false),
        AccountMeta::new_readonly(spl_token::ID, false),
    ];
    let data = jet_bonds::instruction::Withdraw { amount, kind }.data();
    Instruction {
        program_id: jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn place_order_instruction(
    bond_manager_key: &Pubkey,
    user_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    adapter: Option<&Pubkey>,
    order_side: OrderSide,
    order_params: OrderParams,
) -> Instruction {
    let orderbook_market_state_key = Pubkey::find_program_address(
        &[
            b"orderbook_market_state".as_ref(),
            bond_manager_key.as_ref(),
        ],
        &jet_bonds::ID,
    )
    .0;

    let orderbook_user_account_key = Pubkey::find_program_address(
        &[
            b"orderbook_user",
            bond_manager_key.as_ref(),
            user_key.as_ref(),
        ],
        &jet_bonds::ID,
    )
    .0;
    let mut accounts = vec![
        AccountMeta::new(orderbook_user_account_key, false),
        AccountMeta::new_readonly(*user_key, true),
        AccountMeta::new_readonly(*bond_manager_key, false),
        AccountMeta::new(orderbook_market_state_key, false),
        AccountMeta::new(*event_queue_key, false),
        AccountMeta::new(*bids_key, false),
        AccountMeta::new(*asks_key, false),
    ];
    if let Some(key) = adapter {
        accounts.push(AccountMeta::new(*key, false));
    }
    let data = {
        jet_bonds::instruction::PlaceOrder {
            side: order_side,
            params: order_params,
        }
        .data()
    };
    Instruction {
        program_id: jet_bonds::ID,
        accounts,
        data,
    }
}

pub fn place_order_authorized_instruction(
    program_id: Pubkey,
    bond_manager: Pubkey,
    event_queue: Pubkey,
    bids: Pubkey,
    asks: Pubkey,
    user: Pubkey,
    order_side: OrderSide,
    order_params: OrderParams,
) -> Instruction {
    let orderbook_user_account = pda(&[b"orderbook_user", bond_manager.as_ref(), user.as_ref()]);

    Instruction {
        program_id,
        accounts: jet_bonds::accounts::PlaceOrderAuthorized {
            base_accounts: jet_bonds::accounts::PlaceOrder {
                orderbook_user_account,
                user,
                bond_manager,
                orderbook_market_state: pda(&[b"orderbook_market_state", bond_manager.as_ref()]),
                event_queue,
                bids,
                asks,
            },
            claims: pda(&[seeds::CLAIM_NOTES, orderbook_user_account.as_ref()]),
            claims_mint: pda(&[seeds::CLAIM_NOTES, bond_manager.as_ref()]),
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: jet_bonds::instruction::PlaceOrderAuthorized {
            side: order_side,
            params: order_params,
        }
        .data(),
    }
}

pub fn cancel_order_instruction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    user_key: &Pubkey,
    orderbook_market_state_key: &Pubkey,
    event_queue_key: &Pubkey,
    bids_key: &Pubkey,
    asks_key: &Pubkey,
    orderbook_user_account_key: &Pubkey,
    order_id: u128,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*orderbook_user_account_key, false),
        AccountMeta::new_readonly(*user_key, true),
        AccountMeta::new_readonly(*bond_manager_key, false),
        AccountMeta::new(*orderbook_market_state_key, false),
        AccountMeta::new(*event_queue_key, false),
        AccountMeta::new(*bids_key, false),
        AccountMeta::new(*asks_key, false),
    ];
    let data = { jet_bonds::instruction::CancelOrder { order_id }.data() };
    Instruction {
        program_id: *jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn consume_events_instruction(
    bond_manager_key: &Pubkey,
    orderbook_market_state_key: &Pubkey,
    event_queue_key: &Pubkey,
    crank_metadata_key: &Pubkey,
    crank_signer_key: &Pubkey,
    payer_key: &Pubkey,
    remaining_accounts: &[&Pubkey],
    num_events: usize,
    seed_bytes: Vec<Vec<u8>>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(*bond_manager_key, false),
        AccountMeta::new(*orderbook_market_state_key, false),
        AccountMeta::new(*event_queue_key, false),
        AccountMeta::new_readonly(*crank_metadata_key, false),
        AccountMeta::new_readonly(*crank_signer_key, true),
        AccountMeta::new(*payer_key, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    accounts.extend(
        remaining_accounts
            .iter()
            .map(|a| AccountMeta::new(**a, false)),
    );
    let data = {
        jet_bonds::instruction::ConsumeEvents {
            num_events: num_events as u32,
            seed_bytes,
        }
        .data()
    };
    Instruction {
        program_id: jet_bonds::ID,
        accounts,
        data,
    }
}

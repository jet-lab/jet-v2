#![allow(clippy::too_many_arguments)]
use anchor_lang::{InstructionData, ToAccountMetas};
use jet_bonds::{
    control::instructions::InitializeBondManagerParams, seeds,
    tickets::instructions::StakeBondTicketsParams,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::Keypair,
    signer::Signer,
    system_program,
    sysvar::SysvarId,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::pda;

pub fn exchange_tokens_instruction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    underlying_token_mint: &Pubkey,
    user_authority_key: &Pubkey,
    amount: u64,
) -> Instruction {
    let underlying_token_vault_key = Pubkey::find_program_address(
        &[
            b"underlying_token_vault".as_ref(),
            bond_manager_key.as_ref(),
        ],
        jet_bonds_pid,
    )
    .0;
    let bond_ticket_mint_key = Pubkey::find_program_address(
        &[b"bond_ticket_mint".as_ref(), bond_manager_key.as_ref()],
        jet_bonds_pid,
    )
    .0;

    let user_bond_ticket_vault_key =
        get_associated_token_address(user_authority_key, &bond_ticket_mint_key);
    let user_underlying_token_vault_key =
        get_associated_token_address(user_authority_key, underlying_token_mint);

    let accounts = jet_bonds::accounts::ExchangeTokens {
        bond_manager: *bond_manager_key,
        underlying_token_vault: underlying_token_vault_key,
        bond_ticket_mint: bond_ticket_mint_key,
        user_bond_ticket_vault: user_bond_ticket_vault_key,
        user_underlying_token_vault: user_underlying_token_vault_key,
        user_authority: *user_authority_key,
        token_program: spl_token::id(),
    }
    .to_account_metas(None);

    let data = jet_bonds::instruction::ExchangeTokens { amount }.data();
    Instruction {
        program_id: *jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn intitialize_bond_ticket_account_instruction(
    jet_bonds_pid: &Pubkey,
    bond_manager_key: &Pubkey,
    recipient_key: &Pubkey,
    payer_key: &Pubkey,
) -> Instruction {
    let bond_ticket_mint_key = Pubkey::find_program_address(
        &[b"bond_ticket_mint".as_ref(), bond_manager_key.as_ref()],
        jet_bonds_pid,
    )
    .0;
    create_associated_token_account(payer_key, recipient_key, &bond_ticket_mint_key)
}

pub fn initialize_bond_manager_instruction(
    jet_bonds_pid: &Pubkey,
    underlying_token_mint_key: &Pubkey,
    program_authority_key: &Pubkey,
    payer_key: &Pubkey,
    oracle: Option<Pubkey>,
    version_tag: u64,
    duration: i64,
    conversion_factor: i8,
    seed: u64,
) -> Instruction {
    let bond_manager_key = pda(&[
        b"bond_manager".as_ref(),
        underlying_token_mint_key.as_ref(),
        seed.to_le_bytes().as_ref(),
    ]);
    let accounts = jet_bonds::accounts::InitializeBondManager {
        bond_manager: bond_manager_key,
        underlying_token_vault: pda(&[
            b"underlying_token_vault".as_ref(),
            bond_manager_key.as_ref(),
        ]),
        underlying_token_mint: *underlying_token_mint_key,
        bond_ticket_mint: pda(&[b"bond_ticket_mint".as_ref(), bond_manager_key.as_ref()]),
        claims: pda(&[seeds::CLAIM_NOTES, bond_manager_key.as_ref()]),
        program_authority: *program_authority_key,
        oracle: oracle.unwrap_or_else(|| Keypair::new().pubkey()), //todo
        payer: *payer_key,
        rent: Rent::id(),
        token_program: spl_token::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    let data = {
        let params = InitializeBondManagerParams {
            version_tag,
            seed,
            conversion_factor,
            duration,
        };

        jet_bonds::instruction::InitializeBondManager { params }.data()
    };

    Instruction {
        program_id: *jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn redeem_ticket_instruction(
    jet_bonds_pid: &Pubkey,
    ticket_key: &Pubkey,
    ticket_holder_key: &Pubkey,
    claimant_token_account_key: &Pubkey,
    underlying_token_vault_key: &Pubkey,
    bond_manager_key: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*ticket_key, false),
        AccountMeta::new(*ticket_holder_key, true),
        AccountMeta::new(*claimant_token_account_key, false),
        AccountMeta::new_readonly(*bond_manager_key, false),
        AccountMeta::new(*underlying_token_vault_key, false),
        AccountMeta::new_readonly(spl_token::ID, false),
    ];
    let data = jet_bonds::instruction::RedeemTicket {}.data();
    Instruction {
        program_id: *jet_bonds_pid,
        accounts,
        data,
    }
}

pub fn stake_bond_tickets_instruction(
    bond_manager_key: &Pubkey,
    ticket_holder_key: &Pubkey,
    payer_key: &Pubkey,
    ticket_seed: Vec<u8>,
    amount: u64,
) -> Instruction {
    let (claim_ticket_key, _) = Pubkey::find_program_address(
        &[
            b"claim_ticket",
            ticket_holder_key.as_ref(),
            bond_manager_key.as_ref(),
            ticket_seed.as_ref(),
        ],
        &jet_bonds::ID,
    );
    let (bond_ticket_mint_key, _) = Pubkey::find_program_address(
        &[b"bond_ticket_mint".as_ref(), bond_manager_key.as_ref()],
        &jet_bonds::ID,
    );
    let bond_ticket_token_account_key =
        get_associated_token_address(ticket_holder_key, &bond_ticket_mint_key);
    let accounts = jet_bonds::accounts::StakeBondTickets {
        claim_ticket: claim_ticket_key,
        bond_manager: *bond_manager_key,
        ticket_holder: *ticket_holder_key,
        bond_ticket_token_account: bond_ticket_token_account_key,
        bond_ticket_mint: bond_ticket_mint_key,
        payer: *payer_key,
        token_program: spl_token::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    let data = {
        let params = StakeBondTicketsParams {
            amount,
            ticket_seed,
        };
        jet_bonds::instruction::StakeBondTickets { params }.data()
    };
    Instruction {
        program_id: jet_bonds::ID,
        accounts,
        data,
    }
}

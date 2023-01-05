use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use jet_margin_pool::ChangeKind;

use super::{margin::derive_position_token_account, margin_pool::MarginPoolIxBuilder};

/// Instruction for using an SPL swap with tokens in a margin pool
#[allow(clippy::too_many_arguments)]
pub fn pool_spl_swap(
    swap_info: &SplSwap,
    _airspace: &Pubkey,
    margin_account: &Pubkey,
    source_token: &Pubkey,
    target_token: &Pubkey,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    minimum_amount_out: u64,
) -> Instruction {
    let pool_source = MarginPoolIxBuilder::new(*source_token);
    let pool_target = MarginPoolIxBuilder::new(*target_token);

    let transit_source_account = get_associated_token_address(margin_account, source_token);
    let transit_destination_account = get_associated_token_address(margin_account, target_token);

    let source_account =
        derive_position_token_account(margin_account, &pool_source.deposit_note_mint);
    let destination_account =
        derive_position_token_account(margin_account, &pool_target.deposit_note_mint);

    let (vault_into, vault_from) = if *source_token == swap_info.token_a {
        (swap_info.token_a_vault, swap_info.token_b_vault)
    } else {
        (swap_info.token_b_vault, swap_info.token_a_vault)
    };

    let accounts = jet_margin_swap::accounts::MarginSplSwap {
        margin_account: *margin_account,
        source_account,
        destination_account,
        transit_source_account,
        transit_destination_account,
        swap_info: jet_margin_swap::accounts::SwapInfo {
            swap_pool: swap_info.address,
            authority: derive_spl_swap_authority(&swap_info.program, &swap_info.address),
            token_mint: swap_info.pool_mint,
            fee_account: swap_info.fee_account,
            swap_program: swap_info.program,
            vault_into,
            vault_from,
        },
        source_margin_pool: jet_margin_swap::accounts::MarginPoolInfo {
            margin_pool: pool_source.address,
            vault: pool_source.vault,
            deposit_note_mint: pool_source.deposit_note_mint,
        },
        destination_margin_pool: jet_margin_swap::accounts::MarginPoolInfo {
            margin_pool: pool_target.address,
            vault: pool_target.vault,
            deposit_note_mint: pool_target.deposit_note_mint,
        },
        margin_pool_program: jet_margin_pool::ID,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_margin_swap::ID,
        data: jet_margin_swap::instruction::MarginSwap {
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        }
        .data(),
        accounts,
    }
}

pub struct SplSwap {
    pub program: Pubkey,
    pub address: Pubkey,
    pub pool_mint: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub fee_account: Pubkey,
}

pub fn derive_spl_swap_authority(program: &Pubkey, pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[pool.as_ref()], program).0
}

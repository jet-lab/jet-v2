use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};

use jet_rewards::{seeds, state::AirdropRecipient, AirdropV2CreateParams};

use super::REWARDS_PROGRAM;

use crate::staking::{
    derive_max_voter_weight_record, derive_stake_account, derive_stake_pool,
    derive_stake_pool_vault, derive_voter_weight_record, STAKING_PROGRAM,
};

pub fn create(
    payer: Pubkey,
    seed: u64,
    token_mint: Pubkey,
    params: AirdropV2CreateParams,
) -> Instruction {
    let airdrop = derive_airdrop(seed);
    let vault = derive_airdrop_vault(&airdrop);

    let accounts = jet_rewards::accounts::AirdropV2Create {
        payer,
        airdrop,
        vault,
        token_mint,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: REWARDS_PROGRAM,
        data: jet_rewards::instruction::AirdropV2Create { params }.data(),
    }
}

pub fn add_recipients(
    airdrop: Pubkey,
    recipients: impl IntoIterator<Item = (Pubkey, u64)>,
) -> Instruction {
    let accounts = jet_rewards::accounts::AirdropV2AddRecipients {
        airdrop,
        authority: Pubkey::default(),
        payer: Pubkey::default(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let data = jet_rewards::instruction::AirdropV2AddRecipients {
        recipients: recipients
            .into_iter()
            .map(|(recipient, amount)| AirdropRecipient { recipient, amount })
            .collect(),
    }
    .data();

    Instruction {
        accounts,
        data,
        program_id: REWARDS_PROGRAM,
    }
}

pub fn set_review(authority: Pubkey, airdrop: Pubkey, reviewer: Pubkey) -> Instruction {
    let accounts =
        jet_rewards::accounts::AirdropV2SetReview { authority, airdrop }.to_account_metas(None);

    Instruction {
        accounts,
        data: jet_rewards::instruction::AidropV2SetReview { reviewer }.data(),
        program_id: REWARDS_PROGRAM,
    }
}

pub fn finalize(authority: Pubkey, airdrop: Pubkey) -> Instruction {
    let accounts = jet_rewards::accounts::AirdropV2Finalize {
        authority,
        airdrop,
        vault: derive_airdrop_vault(&airdrop),
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        data: jet_rewards::instruction::AirdropV2Finalize.data(),
        program_id: REWARDS_PROGRAM,
    }
}

pub fn close(
    payer: Pubkey,
    authority: Pubkey,
    airdrop: Pubkey,
    token_receiver: Pubkey,
) -> Instruction {
    let accounts = jet_rewards::accounts::AirdropV2Close {
        authority,
        airdrop,
        token_receiver,
        receiver: payer,
        vault: derive_airdrop_vault(&airdrop),
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        data: jet_rewards::instruction::AirdropV2Close.data(),
        program_id: REWARDS_PROGRAM,
    }
}

pub fn claim(airdrop: Pubkey, recipient: Pubkey, stake_seed: &str, realm: Pubkey) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &recipient);

    let accounts = jet_rewards::accounts::AirdropV2Claim {
        airdrop,
        recipient,
        stake_pool,
        stake_account,
        vault: derive_airdrop_vault(&airdrop),
        stake_pool_vault: derive_stake_pool_vault(stake_seed),
        max_voter_weight_record: derive_max_voter_weight_record(&realm),
        voter_weight_record: derive_voter_weight_record(&stake_account),
        token_program: spl_token::ID,
        staking_program: STAKING_PROGRAM,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        data: jet_rewards::instruction::AirdropV2Claim.data(),
        program_id: REWARDS_PROGRAM,
    }
}

pub fn derive_airdrop(seed: u64) -> Pubkey {
    Pubkey::find_program_address(&[seeds::AIRDROP, &seed.to_le_bytes()], &REWARDS_PROGRAM).0
}

pub fn derive_airdrop_vault(airdrop: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[seeds::VAULT, airdrop.as_ref()], &REWARDS_PROGRAM).0
}

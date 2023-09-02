use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    system_program,
};

use jet_rewards::{seeds, AirdropAddRecipientsParams, AirdropCreateParams};

use super::REWARDS_PROGRAM;

pub fn create(
    payer: Pubkey,
    token_mint: Pubkey,
    authority: Pubkey,
    storage: Pubkey,
    params: AirdropCreateParams,
) -> Instruction {
    let accounts = jet_rewards::accounts::AirdropCreate {
        payer,
        token_mint,
        authority,
        airdrop: storage,
        reward_vault: derive_reward_vault(&storage),
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_rewards::ID,
        data: jet_rewards::instruction::AirdropCreate { params }.data(),
    }
}

pub fn add_recipients(
    authority: Pubkey,
    airdrop: Pubkey,
    recipients: impl IntoIterator<Item = (Pubkey, u64)>,
    start_index: u64,
) -> Vec<Instruction> {
    const CHUNK_SIZE: usize = 24;

    let mut recipients = recipients.into_iter().collect::<Vec<_>>();
    recipients.sort_by_key(|r| r.0);

    recipients
        .chunks(CHUNK_SIZE)
        .into_iter()
        .enumerate()
        .map(|(n, chunk)| {
            let accounts = jet_rewards::accounts::AirdropAddRecipients { airdrop, authority }
                .to_account_metas(None);

            let data = jet_rewards::instruction::AirdropAddRecipients {
                params: AirdropAddRecipientsParams {
                    start_index: dbg!(start_index + (n * CHUNK_SIZE) as u64),
                    recipients: chunk
                        .into_iter()
                        .map(|(recipient, amount)| jet_rewards::AirdropRecipientParam {
                            recipient: *recipient,
                            amount: *amount,
                        })
                        .collect(),
                },
            }
            .data();

            Instruction {
                accounts,
                program_id: jet_rewards::ID,
                data,
            }
        })
        .collect()
}

pub fn finalize(authority: Pubkey, airdrop: Pubkey) -> Instruction {
    let accounts = jet_rewards::accounts::AirdropFinalize {
        authority,
        airdrop,
        reward_vault: derive_reward_vault(&airdrop),
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        data: jet_rewards::instruction::AirdropFinalize.data(),
        program_id: jet_rewards::ID,
    }
}

pub fn derive_reward_vault(airdrop: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[airdrop.as_ref(), seeds::VAULT.as_ref()], &REWARDS_PROGRAM).0
}

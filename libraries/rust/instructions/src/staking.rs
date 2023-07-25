use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};

use jet_staking::{seeds};

pub use jet_staking::{ID as STAKING_PROGRAM, PoolConfig};

use crate::auth::derive_user_auth;

pub fn init_pool(
    payer: Pubkey,
    authority: Pubkey,
    token_mint: Pubkey,
    seed: &str,
    config: &PoolConfig,
) -> Instruction {
    let accounts = jet_staking::accounts::InitPool {
        payer,
        authority,
        token_mint,
        stake_pool: derive_stake_pool(&seed),
        max_voter_weight_record: derive_max_voter_weight_record(&config.governance_realm),
        stake_collateral_mint: derive_stake_collateral_mint(&seed),
        stake_pool_vault: derive_stake_pool_vault(&seed),
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::InitPool {
            seed: seed.to_owned(),
            config: config.clone(),
        }
        .data(),
    }
}

pub fn init_stake_account(payer: Pubkey, owner: Pubkey, stake_seed: &str) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &owner);

    let accounts = jet_staking::accounts::InitStakeAccount {
        owner,
        payer,
        stake_pool,
        stake_account,
        auth: derive_user_auth(&owner),
        voter_weight_record: derive_voter_weight_record(&stake_account),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::InitStakeAccount.data(),
    }
}

pub fn add_stake(
    payer: Pubkey,
    owner: Pubkey,
    stake_seed: &str,
    realm: Pubkey,
    amount: Option<u64>,
) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &owner);

    let accounts = jet_staking::accounts::AddStake {
        stake_pool,
        stake_account,
        stake_pool_vault: derive_stake_pool_vault(stake_seed),
        voter_weight_record: derive_voter_weight_record(&stake_account),
        max_voter_weight_record: derive_max_voter_weight_record(&realm),
        payer,
        payer_token_account: derive_user_auth(&payer),
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::AddStake { amount }.data(),
    }
}

pub fn unbond_stake(
    payer: Pubkey,
    owner: Pubkey,
    stake_seed: &str,
    realm: Pubkey,
    unbond_seed: u32,
    amount: Option<u64>,
) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &owner);

    let accounts = jet_staking::accounts::UnbondStake {
        owner,
        payer,
        stake_pool,
        stake_account,
        stake_pool_vault: derive_stake_pool_vault(stake_seed),
        voter_weight_record: derive_voter_weight_record(&stake_account),
        max_voter_weight_record: derive_max_voter_weight_record(&realm),
        unbonding_account: derive_unbonding_account(&stake_account, unbond_seed),
        token_owner_record: derive_user_auth(&owner),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::UnbondStake {
            amount,
            seed: unbond_seed,
        }
        .data(),
    }
}

pub fn withdraw_unbonded(
    payer: Pubkey,
    owner: Pubkey,
    stake_seed: &str,
    unbonding_account: Pubkey,
    token_receiver: Pubkey,
) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &owner);

    let accounts = jet_staking::accounts::WithdrawUnbonded {
        owner,
        stake_account,
        stake_pool,
        unbonding_account,
        token_receiver,
        closer: payer,
        stake_pool_vault: derive_stake_pool_vault(stake_seed),
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::WithdrawUnbonded.data(),
    }
}

pub fn cancel_unbond(
    payer: Pubkey,
    owner: Pubkey,
    stake_seed: &str,
    realm: Pubkey,
    unbonding_account: Pubkey,
) -> Instruction {
    let stake_pool = derive_stake_pool(stake_seed);
    let stake_account = derive_stake_account(&stake_pool, &owner);

    let accounts = jet_staking::accounts::CancelUnbond {
        owner,
        stake_pool,
        stake_account,
        unbonding_account,
        receiver: payer,
        voter_weight_record: derive_voter_weight_record(&stake_account),
        max_voter_weight_record: derive_max_voter_weight_record(&realm),
        stake_pool_vault: derive_stake_pool_vault(stake_seed),
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::CancelUnbond.data(),
    }
}

pub fn close_stake_account(payer: Pubkey, owner: Pubkey, stake_account: Pubkey) -> Instruction {
    let accounts = jet_staking::accounts::CloseStakeAccount {
        owner,
        stake_account,
        closer: payer,
        voter_weight_record: derive_voter_weight_record(&stake_account),
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: STAKING_PROGRAM,
        data: jet_staking::instruction::CloseStakeAccount.data(),
    }
}

pub fn derive_stake_pool(seed: &str) -> Pubkey {
    Pubkey::find_program_address(&[seed.as_bytes()], &STAKING_PROGRAM).0
}

pub fn derive_max_voter_weight_record(realm: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[realm.as_ref(), seeds::MAX_VOTE_WEIGHT_RECORD],
        &STAKING_PROGRAM,
    )
    .0
}

pub fn derive_stake_collateral_mint(seed: &str) -> Pubkey {
    Pubkey::find_program_address(&[seed.as_bytes(), seeds::COLLATERAL_MINT], &STAKING_PROGRAM).0
}

pub fn derive_stake_pool_vault(seed: &str) -> Pubkey {
    Pubkey::find_program_address(&[seed.as_bytes(), seeds::VAULT], &STAKING_PROGRAM).0
}

pub fn derive_stake_account(pool: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[pool.as_ref(), owner.as_ref()], &STAKING_PROGRAM).0
}

pub fn derive_voter_weight_record(stake_account: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[seeds::VOTER_WEIGHT_RECORD, stake_account.as_ref()],
        &STAKING_PROGRAM,
    )
    .0
}

pub fn derive_unbonding_account(stake_account: &Pubkey, seed: u32) -> Pubkey {
    Pubkey::find_program_address(
        &[stake_account.as_ref(), seed.to_le_bytes().as_ref()],
        &STAKING_PROGRAM,
    )
    .0
}

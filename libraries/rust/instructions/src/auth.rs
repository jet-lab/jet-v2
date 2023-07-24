use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};

pub use jet_auth::ID as JET_AUTH_PROGRAM;

pub fn create_user_auth(payer: Pubkey, user: Pubkey) -> Instruction {
    let accounts = jet_auth::accounts::CreateUserAuthentication {
        user,
        payer,
        auth: derive_user_auth(&user),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: JET_AUTH_PROGRAM,
        accounts,
        data: jet_auth::instruction::CreateUserAuth.data(),
    }
}

pub fn authenticate(authority: Pubkey, user: Pubkey) -> Instruction {
    let accounts = jet_auth::accounts::Authenticate {
        auth: derive_user_auth(&user),
        authority,
    }
    .to_account_metas(None);

    Instruction {
        program_id: JET_AUTH_PROGRAM,
        accounts,
        data: jet_auth::instruction::Authenticate.data(),
    }
}

pub fn derive_user_auth(user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[user.as_ref()], &JET_AUTH_PROGRAM).0
}

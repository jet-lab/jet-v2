use anchor_lang::prelude::*;

use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anchor_spl::token::Token;
use jet_margin::AdapterResult;
use mock_adapter::accounts;
use mock_adapter::instruction as args;
use solana_sdk::instruction::Instruction;
use solana_sdk::sysvar::SysvarId;

pub fn noop(result: Option<AdapterResult>) -> Instruction {
    Instruction {
        program_id: mock_adapter::id(),
        accounts: accounts::NoAccounts {}.to_account_metas(None),
        data: args::Noop { result }.data(),
    }
}

pub fn init_mint(index: u8, payer: Pubkey) -> (Instruction, Pubkey) {
    let mint = Pubkey::find_program_address(&[&[index]], &mock_adapter::ID).0;
    let ix = Instruction {
        program_id: mock_adapter::id(),
        accounts: accounts::InitMint {
            mint,
            authority: mock_adapter::signer().0,
            payer,
            token_program: Token::id(),
            system_program: System::id(),
            rent: Rent::id(),
        }
        .to_account_metas(None),
        data: args::InitMint { index }.data(),
    };

    (ix, mint)
}

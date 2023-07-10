use anchor_lang::prelude::{Pubkey, Rent};
use anchor_lang::{InstructionData, ToAccountMetas};
use jet_margin_swap::accounts as ix_accounts;
use jet_margin_swap::instruction as ix_data;
use jet_margin_swap::seeds::OPENBOOK_OPEN_ORDERS;
use solana_sdk::instruction::Instruction;
use solana_sdk::system_program::ID as SYSTEM_PROGAM_ID;
use solana_sdk::sysvar::SysvarId;

/// Create an open orders account
pub fn create_open_orders(
    authority: Pubkey,
    market: Pubkey,
    payer: Pubkey,
    program: &Pubkey,
) -> (Instruction, Pubkey) {
    let (open_orders, _) = Pubkey::find_program_address(
        &[OPENBOOK_OPEN_ORDERS, authority.as_ref(), market.as_ref()],
        &jet_margin_swap::id(),
    );
    let accounts = ix_accounts::InitOpenOrders {
        margin_account: authority,
        market,
        payer,
        open_orders,
        dex_program: *program,
        system_program: SYSTEM_PROGAM_ID,
        rent: Rent::id(),
    }
    .to_account_metas(None);

    (
        Instruction {
            program_id: jet_margin_swap::id(),
            accounts,
            data: ix_data::InitOpenbookOpenOrders {}.data(),
        },
        open_orders,
    )
}

/// Close an open orders account
pub fn close_open_orders(
    authority: Pubkey,
    market: Pubkey,
    recipient: Pubkey,
    program: &Pubkey,
) -> Instruction {
    let (open_orders, _) = Pubkey::find_program_address(
        &[OPENBOOK_OPEN_ORDERS, authority.as_ref(), market.as_ref()],
        &jet_margin_swap::id(),
    );
    let accounts = ix_accounts::CloseOpenOrders {
        margin_account: authority,
        market,
        open_orders,
        dex_program: *program,
        destination: recipient,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_margin_swap::id(),
        accounts,
        data: ix_data::CloseOpenbookOpenOrders {}.data(),
    }
}

pub fn derive_open_orders(market: &Pubkey, authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[OPENBOOK_OPEN_ORDERS, authority.as_ref(), market.as_ref()],
        &jet_margin_swap::id(),
    )
    .0
}

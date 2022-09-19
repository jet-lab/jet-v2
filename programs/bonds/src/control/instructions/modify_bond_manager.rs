use std::io::Write;

use anchor_lang::prelude::*;
use jet_metadata::ControlAuthority;

use crate::{control::state::BondManager, BondsError};

#[derive(Accounts)]
pub struct ModifyBondManager<'info> {
    /// The `BondManager` manages asset tokens for a particular bond duration
    #[account(
        mut,
        has_one = program_authority,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The authority to create markets, which must sign
    #[account(signer)]
    pub program_authority: Box<Account<'info, ControlAuthority>>,
}

pub fn handler(ctx: Context<ModifyBondManager>, data: Vec<u8>, offset: usize) -> Result<()> {
    let info = ctx.accounts.bond_manager.to_account_info();
    let buffer = &mut info.data.borrow_mut();

    (&mut buffer[(offset + 8)..])
        .write_all(&data)
        .map_err(|_| BondsError::IoError)?;

    Ok(())
}

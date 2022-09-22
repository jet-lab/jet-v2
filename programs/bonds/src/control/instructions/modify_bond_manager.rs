use std::io::Write;

use anchor_lang::prelude::*;

use crate::{control::state::BondManager, BondsError};

#[derive(Accounts)]
pub struct ModifyBondManager<'info> {
    /// The `BondManager` manages asset tokens for a particular bond duration
    #[account(mut, has_one = airspace @ BondsError::WrongAirspace)]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ BondsError::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ModifyBondManager>, data: Vec<u8>, offset: usize) -> Result<()> {
    let info = ctx.accounts.bond_manager.to_account_info();
    let buffer = &mut info.data.borrow_mut();

    (&mut buffer[(offset + 8)..])
        .write_all(&data)
        .map_err(|_| BondsError::IoError)?;

    Ok(())
}

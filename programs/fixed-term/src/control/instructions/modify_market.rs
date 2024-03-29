use std::io::Write;

use anchor_lang::prelude::*;

use jet_airspace::state::Airspace;

use crate::{control::state::Market, FixedTermErrorCode};

#[derive(Accounts)]
pub struct ModifyMarket<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(mut, has_one = airspace @ FixedTermErrorCode::WrongAirspace)]
    pub market: AccountLoader<'info, Market>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[cfg_attr(not(feature = "testing"), account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization))]
    pub airspace: Account<'info, Airspace>,
}

pub fn handler(ctx: Context<ModifyMarket>, data: Vec<u8>, offset: u32) -> Result<()> {
    if cfg!(not(feature = "testing")) {
        return Ok(());
    }

    let info = ctx.accounts.market.to_account_info();
    let buffer = &mut info.data.borrow_mut();

    (&mut buffer[(offset as usize + 8)..])
        .write_all(&data)
        .map_err(|_| FixedTermErrorCode::IoError)?;

    Ok(())
}

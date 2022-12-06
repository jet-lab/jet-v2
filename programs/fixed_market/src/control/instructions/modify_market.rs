use std::io::Write;

use anchor_lang::prelude::*;

use crate::{control::state::Market, ErrorCode};

#[derive(Accounts)]
pub struct ModifyMarket<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(mut, has_one = airspace @ ErrorCode::WrongAirspace)]
    pub market: AccountLoader<'info, Market>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ ErrorCode::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ModifyMarket>, data: Vec<u8>, offset: usize) -> Result<()> {
    let info = ctx.accounts.market.to_account_info();
    let buffer = &mut info.data.borrow_mut();

    (&mut buffer[(offset + 8)..])
        .write_all(&data)
        .map_err(|_| ErrorCode::IoError)?;

    Ok(())
}

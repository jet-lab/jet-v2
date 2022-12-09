use anchor_lang::prelude::*;

use crate::control::state::CrankAuthorization;

#[derive(Accounts)]
pub struct RevokeCrank<'info> {
    /// The account containing the metadata for the key
    #[account(mut, close = receiver)]
    pub metadata_account: Account<'info, CrankAuthorization>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority)] todo
    pub airspace: AccountInfo<'info>,

    #[account(mut)]
    pub receiver: AccountInfo<'info>,
}

pub fn handler(_: Context<RevokeCrank>) -> Result<()> {
    Ok(())
}

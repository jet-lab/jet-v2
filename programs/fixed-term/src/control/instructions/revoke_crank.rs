use anchor_lang::prelude::*;

use jet_airspace::state::Airspace;

#[cfg(not(feature = "testing"))]
use crate::FixedTermErrorCode;

use crate::control::state::CrankAuthorization;

#[derive(Accounts)]
pub struct RevokeCrank<'info> {
    /// The account containing the metadata for the key
    #[account(mut, close = receiver)]
    pub metadata_account: Account<'info, CrankAuthorization>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[cfg_attr(not(feature = "testing"), account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization))]
    pub airspace: Account<'info, Airspace>,

    #[account(mut)]
    pub receiver: AccountInfo<'info>,
}

pub fn handler(_: Context<RevokeCrank>) -> Result<()> {
    Ok(())
}

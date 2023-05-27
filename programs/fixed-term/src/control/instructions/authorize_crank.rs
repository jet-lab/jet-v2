use anchor_lang::prelude::*;

use jet_airspace::state::Airspace;

use crate::control::state::{CrankAuthorization, Market};
use crate::FixedTermErrorCode;

#[derive(Accounts)]
pub struct AuthorizeCrank<'info> {
    /// The crank signer pubkey
    pub crank: AccountInfo<'info>,

    /// The account containing the metadata for the key
    #[account(
        init,
        seeds = [
            crate::seeds::CRANK_AUTHORIZATION,
            market.key().as_ref(),
            crank.key.as_ref()
        ],
        bump,
        space = std::mem::size_of::<CrankAuthorization>() + 8,
        payer = payer
    )]
    pub crank_authorization: Account<'info, CrankAuthorization>,

    /// The market this signer is authorized to send instructions to
    #[account(has_one = airspace @ FixedTermErrorCode::WrongAirspace)]
    pub market: AccountLoader<'info, Market>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[cfg_attr(not(feature = "testing"), account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization))]
    pub airspace: Account<'info, Airspace>,

    /// The address paying the rent for the account
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AuthorizeCrank>) -> Result<()> {
    *ctx.accounts.crank_authorization = CrankAuthorization {
        crank: ctx.accounts.crank.key(),
        airspace: ctx.accounts.airspace.key(),
        market: ctx.accounts.market.key(),
    };
    Ok(())
}

use anchor_lang::prelude::*;

use crate::control::state::CrankAuthorization;

#[derive(Accounts)]
pub struct AuthorizeCrank<'info> {
    /// The crank signer pubkey
    pub crank: AccountInfo<'info>,

    /// The account containing the metadata for the key
    #[account(
        init,
        seeds = [
            crate::seeds::CRANK_AUTHORIZATION,
            crank.key.as_ref()
        ],
        bump,
        space = std::mem::size_of::<CrankAuthorization>() + 8,
        payer = payer
    )]
    pub crank_authorization: Account<'info, CrankAuthorization>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ BondsError::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,

    /// The address paying the rent for the account
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AuthorizeCrank>) -> Result<()> {
    ctx.accounts.crank_authorization.crank = ctx.accounts.crank.key();
    Ok(())
}

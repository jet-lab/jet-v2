use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use jet_margin::{AdapterResult, PositionChange};

use crate::{
    margin::state::{return_to_margin, MarginUser},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct RegisterPosition<'info> {
    /// The account tracking information related to this particular user
    pub margin_user: Account<'info, MarginUser>,

    /// The signing authority for this user account
    pub margin_account: Signer<'info>,

    /// The token account that can be used by margin to represent a user's position in fixed term
    pub position_token_account: Account<'info, TokenAccount>,

    /// Token metadata account needed by the margin program to register the position
    pub position_metadata: AccountInfo<'info>,
}

pub fn handler(ctx: Context<RegisterPosition>) -> Result<()> {
    let user = &ctx.accounts.margin_user;
    let pta = &ctx.accounts.position_token_account;

    require!(
        pta.key() == user.claims || pta.key() == user.ticket_collateral,
        FixedTermErrorCode::InvalidPosition
    );

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![(pta.mint, vec![PositionChange::Register(pta.key())])],
        },
    )
}

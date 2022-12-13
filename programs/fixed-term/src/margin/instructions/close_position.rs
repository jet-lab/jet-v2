use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use jet_margin::{AdapterResult, PositionChange};

use crate::{
    margin::state::{return_to_margin, MarginUser},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    /// The account tracking information related to this particular user
    pub borrower_account: Account<'info, MarginUser>,

    /// The signing authority for this user account
    pub margin_account: Signer<'info>,

    /// The token account that can be used by margin to represent a user's position in fixed term
    pub position_token_account: Account<'info, TokenAccount>,
}

pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    let user = &ctx.accounts.borrower_account;
    let pta = &ctx.accounts.position_token_account;

    require!(
        pta.key() == user.claims || pta.key() == user.ticket_collateral,
        FixedTermErrorCode::InvalidPosition
    );

    if pta.key() == user.claims {
        require!(pta.amount == 0, FixedTermErrorCode::NonZeroDebt);
    }

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![(pta.mint, vec![PositionChange::Close(pta.key())])],
        },
    )
}

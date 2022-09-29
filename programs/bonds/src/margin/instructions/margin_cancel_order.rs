use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{margin::state::MarginUser, orderbook::state::*, BondsError};

#[derive(Accounts)]
pub struct MarginCancelOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
    )]
    pub borrower_account: Box<Account<'info, MarginUser>>,

    /// The signing authority for this user account
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: constraint
    #[account(
        mut,
        constraint =
            borrower_account.claims == claims.key()
            @ BondsError::WrongClaimAccount
    )]
    pub claims: UncheckedAccount<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    #[account(mut)]
    pub claims_mint: UncheckedAccount<'info>,

    pub orderbook_mut: OrderbookMut<'info>,

    pub token_program: Program<'info, Token>,
}

/// remove order from the book
pub fn handler(ctx: Context<MarginCancelOrder>, order_id: u128) -> Result<()> {
    let (_, callback_flags, _) = ctx
        .accounts
        .orderbook_mut
        .cancel_order(order_id, ctx.accounts.borrower_account.key())?;

    require!(
        callback_flags.contains(CallbackFlags::MARGIN),
        BondsError::UnauthorizedCaller
    );

    Ok(())
}

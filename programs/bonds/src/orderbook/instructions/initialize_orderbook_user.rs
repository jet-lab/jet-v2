use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::state::BondManager, events::OrderbookUserInitialized,
    orderbook::state::user::OrderbookUser, seeds, utils::init, BondsError,
};

#[derive(Accounts)]
pub struct InitializeOrderbookUser<'info> {
    /// The account tracking information related to this particular user
    #[account(
        init,
        seeds = [
            seeds::ORDERBOOK_USER,
            bond_manager.key().as_ref(),
            user.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<OrderbookUser>(),
    )]
    pub orderbook_user_account: Box<Account<'info, OrderbookUser>>,

    /// The signing authority for this user account
    pub user: Signer<'info>,

    /// The Boheader account
    #[account(has_one = claims_mint @ BondsError::WrongClaimMint)]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// Token account used by the margin program to track the debt
    /// that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            orderbook_user_account.key().as_ref(),
        ],
        bump,
        token::mint = claims_mint,
        token::authority = bond_manager,
        payer = payer)]
    pub claims: Account<'info, TokenAccount>,
    pub claims_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<InitializeOrderbookUser>) -> Result<()> {
    let user = &mut ctx.accounts.orderbook_user_account;

    init! {
        user = OrderbookUser {
            user: ctx.accounts.user.key(),
            bond_manager: ctx.accounts.bond_manager.key(),
            claims: ctx.accounts.claims.key(),
        } ignoring {
            event_adapter,
            bond_tickets_stored,
            underlying_token_stored,
            outstanding_obligations,
            debt,
            nonce,
        }
    }

    emit!(OrderbookUserInitialized {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        owner: ctx.accounts.user.key(),
    });

    Ok(())
}

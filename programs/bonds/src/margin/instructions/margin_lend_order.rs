use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    margin::state::MarginUser, orderbook::state::*, serialization::RemainingAccounts,
    utils::mint_to, BondsError,
};

#[derive(Accounts)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = collateral @ BondsError::WrongCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The margin account for this borrow order
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral_mint: AccountInfo<'info>,

    pub orderbook_mut: OrderbookMut<'info>,

    #[account(
        constraint = lend.lender_mint() == orderbook_mut.underlying_mint() @ BondsError::WrongUnderlyingTokenMint,
        constraint = lend.vault() == orderbook_mut.vault() @ BondsError::WrongVault,
    )]
    pub lend: Lend<'info>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.margin_account.key(),
        Side::Bid,
        params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;
    ctx.accounts.lend.lend(
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_account.to_account_info(),
        &seed,
        callback_info,
        &order_summary,
        &ctx.accounts.orderbook_mut.bond_manager,
    )?;
    mint_to!(
        ctx,
        collateral_mint,
        collateral,
        order_summary.quote_combined()?,
        orderbook_mut
    )?;
    emit!(crate::events::MarginLend {
        bond_market: ctx.accounts.orderbook_mut.bond_manager.key(),
        margin_account: ctx.accounts.margin_account.key(),
        lender: ctx.accounts.margin_user.key(),
        order_summary: order_summary.summary(),
    });

    Ok(())
}

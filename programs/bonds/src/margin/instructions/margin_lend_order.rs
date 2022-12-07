use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use jet_program_proc_macros::BondTokenManager;

use crate::{
    bond_token_manager::BondTokenManager,
    events::AssetsUpdated,
    margin::state::MarginUser,
    orderbook::{
        instructions::lend_order::*,
        state::{CallbackFlags, OrderParams},
    },
    serialization::RemainingAccounts,
    BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        constraint = margin_user.margin_account.key() == inner.authority.key(),
        has_one = collateral @ BondsError::WrongCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral_mint: AccountInfo<'info>,

    #[bond_manager(orderbook_mut)]
    #[token_program]
    pub inner: LendOrder<'info>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let (callback_info, order_summary) = ctx.accounts.inner.orderbook_mut.place_order(
        ctx.accounts.inner.authority.key(),
        Side::Bid,
        params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::MARGIN
            | if params.auto_stake {
                CallbackFlags::AUTO_STAKE
            } else {
                CallbackFlags::empty()
            },
    )?;
    let staked = ctx.accounts.inner.lend(
        ctx.accounts.margin_user.key(),
        &seed,
        callback_info,
        &order_summary,
    )?;
    ctx.accounts.margin_user.assets.stake_tickets(staked)?;
    ctx.mint(
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        staked + order_summary.quote_posted()?,
    )?;
    emit!(crate::events::OrderPlaced {
        bond_manager: ctx.accounts.inner.orderbook_mut.bond_manager.key(),
        authority: ctx.accounts.inner.authority.key(),
        margin_user: Some(ctx.accounts.margin_user.key()),
        order_summary: order_summary.summary(),
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
        limit_price: params.limit_price,
        order_type: crate::events::OrderType::MarginLend,
    });
    emit!(AssetsUpdated::from((
        &ctx.accounts.margin_user.assets,
        ctx.accounts.margin_user.key()
    )));

    Ok(())
}

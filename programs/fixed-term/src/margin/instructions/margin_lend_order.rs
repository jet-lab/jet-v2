use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser,
    market_token_manager::MarketTokenManager,
    orderbook::{
        instructions::lend_order::*,
        state::{CallbackFlags, OrderParams},
    },
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        constraint = margin_user.margin_account.key() == inner.authority.key(),
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market(orderbook_mut)]
    #[token_program]
    pub inner: LendOrder<'info>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams) -> Result<()> {
    let user = &mut ctx.accounts.margin_user;

    let (callback_info, order_summary) = ctx.accounts.inner.orderbook_mut.place_order(
        ctx.accounts.inner.authority.key(),
        Side::Bid,
        params,
        user.key(),
        user.key(),
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
        user.key(),
        ctx.accounts.inner.payer.key(),
        &user.assets.next_new_deposit_seqno().to_le_bytes(),
        user.assets.next_new_deposit_seqno(),
        callback_info,
        &order_summary,
    )?;
    ctx.accounts.margin_user.assets.new_deposit(staked)?;
    ctx.mint(
        &ctx.accounts.ticket_collateral_mint,
        &ctx.accounts.ticket_collateral,
        staked + order_summary.quote_posted()?,
    )?;
    emit!(crate::events::OrderPlaced {
        market: ctx.accounts.inner.orderbook_mut.market.key(),
        authority: ctx.accounts.inner.authority.key(),
        margin_user: Some(ctx.accounts.margin_user.key()),
        order_tag: callback_info.order_tag.as_u128(),
        order_summary: order_summary.summary(),
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
        limit_price: params.limit_price,
        order_type: crate::events::OrderType::MarginLend,
    });
    ctx.accounts.margin_user.emit_asset_balances();

    Ok(())
}

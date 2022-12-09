use agnostic_orderbook::instruction::resume_matching;
use anchor_lang::prelude::*;

use crate::{
    control::{events::ToggleOrderMatching, state::Market},
    orderbook::state::CallbackInfo,
    ErrorCode,
};

#[derive(Accounts)]
pub struct ResumeOrderMatching<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(
        has_one = airspace @ ErrorCode::WrongAirspace,
        has_one = orderbook_market_state @ ErrorCode::WrongMarketState,
        has_one = bids @ ErrorCode::WrongBids,
        has_one = asks @ ErrorCode::WrongAsks,
        has_one = event_queue @ ErrorCode::WrongEventQueue,
    )]
    pub market: AccountLoader<'info, Market>,

    // aaob accounts
    /// CHECK: handled by has_one on market
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by has_one on market
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by has_one on market
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by has_one on market
    #[account(mut)]
    pub asks: AccountInfo<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ ErrorCode::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ResumeOrderMatching>) -> Result<()> {
    let accounts = resume_matching::Accounts {
        market: &ctx.accounts.orderbook_market_state,
        asks: &ctx.accounts.asks,
        bids: &ctx.accounts.bids,
        event_queue: &ctx.accounts.event_queue,
    };
    let params = resume_matching::Params {};
    resume_matching::process::<CallbackInfo>(ctx.program_id, accounts, params)?;

    emit!(ToggleOrderMatching {
        market: ctx.accounts.market.key(),
        is_orderbook_paused: false
    });

    Ok(())
}

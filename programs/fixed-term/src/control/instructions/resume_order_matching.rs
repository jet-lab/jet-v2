use anchor_lang::prelude::*;

use agnostic_orderbook::{
    instruction::resume_matching,
    state::{market_state::MarketState, AccountTag},
};
use jet_airspace::state::Airspace;

use crate::{
    control::{events::ToggleOrderMatching, state::Market},
    orderbook::state::CallbackInfo,
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct ResumeOrderMatching<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(
        has_one = airspace @ FixedTermErrorCode::WrongAirspace,
        has_one = orderbook_market_state @ FixedTermErrorCode::WrongMarketState,
        has_one = bids @ FixedTermErrorCode::WrongBids,
        has_one = asks @ FixedTermErrorCode::WrongAsks,
        has_one = event_queue @ FixedTermErrorCode::WrongEventQueue,
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
    #[cfg_attr(not(feature = "testing"), account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization))]
    pub airspace: Account<'info, Airspace>,
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

    let mut market_data = ctx.accounts.orderbook_market_state.data.borrow_mut();
    let market_state = MarketState::from_buffer(&mut market_data, AccountTag::Market)?;

    if market_state.pause_matching == 0 {
        emit!(ToggleOrderMatching {
            market: ctx.accounts.market.key(),
            is_orderbook_paused: false
        });
    }

    Ok(())
}

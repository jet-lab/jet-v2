use agnostic_orderbook::instruction::resume_matching;
use anchor_lang::prelude::*;

use crate::{
    control::{events::ToggleOrderMatching, state::BondManager},
    orderbook::state::CallbackInfo,
    BondsError,
};

#[derive(Accounts)]
pub struct ResumeOrderMatching<'info> {
    /// The `BondManager` manages asset tokens for a particular bond duration
    #[account(
        has_one = airspace @ BondsError::WrongAirspace,
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = bids @ BondsError::WrongBids,
        has_one = asks @ BondsError::WrongAsks,
        has_one = event_queue @ BondsError::WrongEventQueue,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    // aaob accounts
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub asks: AccountInfo<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ BondsError::WrongAirspaceAuthorization)] fixme airspace
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
        bond_manager: ctx.accounts.bond_manager.key(),
        is_orderbook_paused: false
    });

    Ok(())
}

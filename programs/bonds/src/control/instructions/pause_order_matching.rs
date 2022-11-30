use anchor_lang::prelude::*;

use crate::{
    control::{events::ToggleOrderMatching, state::MarketManager},
    orderbook::state::CallbackInfo,
    ErrorCode,
};

#[derive(Accounts)]
pub struct PauseOrderMatching<'info> {
    /// The `MarketManager` manages asset tokens for a particular market tenor
    #[account(
        has_one = orderbook_market_state @ ErrorCode::WrongMarketState,
        has_one = airspace @ ErrorCode::WrongAirspace,
    )]
    pub market_manager: AccountLoader<'info, MarketManager>,

    /// CHECK: has_one on market manager
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ ErrorCode::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,
}

pub fn handler(ctx: Context<PauseOrderMatching>) -> Result<()> {
    let accounts = agnostic_orderbook::instruction::pause_matching::Accounts {
        market: &ctx.accounts.orderbook_market_state,
    };
    let params = agnostic_orderbook::instruction::pause_matching::Params {};
    agnostic_orderbook::instruction::pause_matching::process::<CallbackInfo>(
        ctx.program_id,
        accounts,
        params,
    )?;

    emit!(ToggleOrderMatching {
        market_manager: ctx.accounts.market_manager.key(),
        is_orderbook_paused: true
    });

    Ok(())
}

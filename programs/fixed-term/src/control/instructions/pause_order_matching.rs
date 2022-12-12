use anchor_lang::prelude::*;

use crate::{
    control::{events::ToggleOrderMatching, state::Market},
    orderbook::state::CallbackInfo,
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct PauseOrderMatching<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(
        has_one = orderbook_market_state @ FixedTermErrorCode::WrongMarketState,
        has_one = airspace @ FixedTermErrorCode::WrongAirspace,
    )]
    pub market: AccountLoader<'info, Market>,

    /// CHECK: has_one on market
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization)] fixme airspace
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
        market: ctx.accounts.market.key(),
        is_orderbook_paused: true
    });

    Ok(())
}

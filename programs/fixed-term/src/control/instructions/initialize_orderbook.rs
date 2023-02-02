use anchor_lang::prelude::*;

use agnostic_orderbook::state::market_state::MarketState;
use jet_airspace::state::Airspace;

#[cfg(not(feature = "testing"))]
use crate::FixedTermErrorCode;

use crate::{
    control::{events::OrderbookInitialized, state::Market},
    orderbook::state::{CallbackInfo, TICK_SIZE},
    seeds, FixedTermErrorCode,
};

/// Parameters necessary for orderbook initialization
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct InitializeOrderbookParams {
    /// The minimum order size that can be inserted into the orderbook after matching.
    pub min_base_order_size: u64,
}

/// Initialization of the orderbook for a given asset and tenor
#[derive(Accounts)]
pub struct InitializeOrderbook<'info> {
    /// The `Market` account tracks global information related to this particular Jet market
    #[account(
        mut,
        has_one = airspace @ FixedTermErrorCode::WrongAirspace,
    )]
    pub market: AccountLoader<'info, Market>,

    /// AOB market state
    #[account(init,
              seeds = [
                  seeds::ORDERBOOK_MARKET_STATE,
                  market.key().as_ref()
              ],
              bump,
              space = 8 + MarketState::LEN,
              payer = payer,
    )]
    pub orderbook_market_state: AccountInfo<'info>,

    /// AOB market event queue
    ///
    /// Must be initialized
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    /// AOB market bids
    #[account(mut)]
    pub bids: AccountInfo<'info>,

    /// AOB market asks
    #[account(mut)]
    pub asks: AccountInfo<'info>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[cfg_attr(not(feature = "testing"), account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization))]
    pub airspace: Account<'info, Airspace>,

    /// The account paying rent for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeOrderbook>, params: InitializeOrderbookParams) -> Result<()> {
    let InitializeOrderbookParams {
        min_base_order_size,
    } = params;

    // assign the Jet market header data
    let mut manager = ctx.accounts.market.load_mut()?;
    manager.orderbook_market_state = ctx.accounts.orderbook_market_state.key();
    manager.event_queue = ctx.accounts.event_queue.key();
    manager.asks = ctx.accounts.asks.key();
    manager.bids = ctx.accounts.bids.key();

    // initialize the asset agnostic orderbook
    let program_id = ctx.program_id;
    let orderbook_accounts = agnostic_orderbook::instruction::create_market::Accounts {
        market: &ctx.accounts.orderbook_market_state.to_account_info(),
        event_queue: &ctx.accounts.event_queue.to_account_info(),
        bids: &ctx.accounts.bids.to_account_info(),
        asks: &ctx.accounts.asks.to_account_info(),
    };
    let orderbook_params = agnostic_orderbook::instruction::create_market::Params {
        min_base_order_size,
        tick_size: TICK_SIZE,
    };

    agnostic_orderbook::instruction::create_market::process::<CallbackInfo>(
        program_id,
        orderbook_accounts,
        orderbook_params,
    )?;

    emit!(OrderbookInitialized {
        market: ctx.accounts.market.key(),
        orderbook_market_state: manager.orderbook_market_state,
        event_queue: manager.event_queue,
        bids: manager.bids,
        asks: manager.asks,
        min_base_order_size,
        tick_size: TICK_SIZE
    });

    Ok(())
}

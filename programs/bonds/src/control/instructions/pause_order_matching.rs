use anchor_lang::prelude::*;

use crate::{control::state::BondManager, orderbook::state::CallbackInfo, BondsError};

#[derive(Accounts)]
pub struct PauseOrderMatching<'info> {
    /// The `BondManager` manages asset tokens for a particular bond duration
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = program_authority @ BondsError::WrongProgramAuthority,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// CHECK: has_one on bond manager
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,

    /// The controlling signer for this program
    pub program_authority: Signer<'info>,
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
    Ok(())
}

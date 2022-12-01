use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::MarketTokenManager;

use crate::{control::state::MarketManager, market_token_manager::MarketTokenManager, ErrorCode};

#[derive(Accounts, MarketTokenManager)]
pub struct WithdrawFees<'info> {
    #[account(mut,
        has_one = fee_destination @ ErrorCode::WrongFeeDestination,
        has_one = underlying_token_vault @ ErrorCode::WrongVault,
    )]
    pub market_manager: AccountLoader<'info, MarketManager>,

    #[account(mut)]
    pub fee_destination: AccountInfo<'info>,

    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawFees>) -> Result<()> {
    let mut manager = ctx.accounts.market_manager.load_mut()?;
    ctx.accounts.withdraw(
        &ctx.accounts.underlying_token_vault,
        &ctx.accounts.fee_destination,
        manager.collected_fees,
    )?;
    manager.collected_fees = 0;

    Ok(())
}

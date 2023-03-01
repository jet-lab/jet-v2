use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::{events::FeesWithdrawn, state::Market},
    market_token_manager::MarketTokenManager,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct WithdrawFees<'info> {
    #[account(mut,
        has_one = fee_destination @ FixedTermErrorCode::WrongFeeDestination,
        has_one = fee_vault @ FixedTermErrorCode::WrongVault,
    )]
    pub market: AccountLoader<'info, Market>,

    #[account(mut)]
    pub fee_destination: AccountInfo<'info>,

    #[account(mut)]
    pub fee_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawFees>) -> Result<()> {
    let collected_fees = ctx.accounts.fee_vault.amount;
    ctx.accounts.withdraw(
        &ctx.accounts.fee_vault.to_account_info(),
        &ctx.accounts.fee_destination,
        collected_fees,
    )?;

    emit!(FeesWithdrawn {
        market: ctx.accounts.market.key(),
        fee_destination: ctx.accounts.fee_destination.key(),
        collected_fees,
    });

    Ok(())
}

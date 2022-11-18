use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::BondTokenManager;

use crate::{bond_token_manager::BondTokenManager, control::state::BondManager, BondsError};

#[derive(Accounts, BondTokenManager)]
pub struct WithdrawFees<'info> {
    #[account(mut,
        has_one = fee_destination @ BondsError::WrongFeeDestination,
        has_one = underlying_token_vault @ BondsError::WrongVault,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    #[account(mut)]
    pub fee_destination: AccountInfo<'info>,

    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawFees>) -> Result<()> {
    let mut manager = ctx.accounts.bond_manager.load_mut()?;
    ctx.accounts.withdraw(
        &ctx.accounts.underlying_token_vault,
        &ctx.accounts.fee_destination,
        manager.collected_fees,
    )?;
    manager.collected_fees = 0;

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market, margin::state::MarginUser, market_token_manager::MarketTokenManager,
    ErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct Settle<'info> {
    /// The account tracking information related to this particular user
    #[account(mut,
        has_one = market @ ErrorCode::UserNotInMarket,
        has_one = claims @ ErrorCode::WrongClaimAccount,
        has_one = collateral @ ErrorCode::WrongCollateralAccount,
        has_one = underlying_settlement @ ErrorCode::WrongUnderlyingSettlementAccount,
        has_one = ticket_settlement @ ErrorCode::WrongTicketSettlementAccount,
    )]
    pub margin_user: Account<'info, MarginUser>,

    /// The `Market` account tracks global information related to this particular fixed term market
    #[account(
        has_one = underlying_token_vault @ ErrorCode::WrongVault,
        has_one = ticket_mint @ ErrorCode::WrongOracle,
        has_one = claims_mint @ ErrorCode::WrongClaimMint,
        has_one = collateral_mint @ ErrorCode::WrongCollateralMint,
    )]
    pub market: AccountLoader<'info, Market>,

    /// SPL token program
    pub token_program: Program<'info, Token>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub claims: Account<'info, TokenAccount>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: token program checks it
    #[account(mut)]
    pub claims_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub collateral: Account<'info, TokenAccount>,

    /// CHECK: token program checks it
    #[account(mut)]
    pub collateral_mint: UncheckedAccount<'info>,

    /// CHECK: token program checks it
    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,
    /// CHECK: token program checks it
    #[account(mut)]
    pub ticket_mint: AccountInfo<'info>,
    /// CHECK: token program checks it
    #[account(mut)]
    pub underlying_settlement: AccountInfo<'info>,
    /// CHECK: token program checks it
    #[account(mut)]
    pub ticket_settlement: AccountInfo<'info>,
}

pub fn handler(ctx: Context<Settle>) -> Result<()> {
    let claim_balance = ctx.accounts.claims.amount;
    let ctokens_held = ctx.accounts.collateral.amount;
    let assets = &ctx.accounts.margin_user.assets;
    let debt = ctx.accounts.margin_user.debt.total();
    let ctokens_deserved = assets.collateral()?;

    // Notify margin of the current debt owed to Jet markets
    if claim_balance > debt {
        ctx.burn_notes(
            &ctx.accounts.claims_mint,
            &ctx.accounts.claims,
            claim_balance - debt,
        )?;
    }
    if claim_balance < debt {
        ctx.mint(
            &ctx.accounts.claims_mint,
            &ctx.accounts.claims,
            debt - claim_balance,
        )?;
    }

    // Notify margin of the amount of collateral that will in the custody of
    // tickets after this settlement
    if ctokens_held > ctokens_deserved {
        ctx.burn_notes(
            &ctx.accounts.collateral_mint,
            &ctx.accounts.collateral,
            ctokens_held - ctokens_deserved,
        )?;
    }
    if ctokens_held < ctokens_deserved {
        ctx.mint(
            &ctx.accounts.collateral_mint,
            &ctx.accounts.collateral,
            ctokens_deserved - ctokens_held,
        )?;
    }

    // Disburse entitled funds due to fills
    ctx.mint(
        &ctx.accounts.ticket_mint,
        &ctx.accounts.ticket_settlement,
        assets.entitled_tickets,
    )?;
    ctx.withdraw(
        &ctx.accounts.underlying_token_vault,
        &ctx.accounts.underlying_settlement,
        assets.entitled_tokens,
    )?;

    // Update margin user assets to reflect the settlement
    ctx.accounts.margin_user.assets.entitled_tickets = 0;
    ctx.accounts.margin_user.assets.entitled_tokens = 0;

    Ok(())
}

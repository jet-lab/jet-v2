use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    control::state::BondManager,
    margin::state::MarginUser,
    utils::{burn, mint_to, withdraw},
    BondsError,
};

#[derive(Accounts)]
pub struct Settle<'info> {
    /// The account tracking information related to this particular user
    #[account(
        has_one = bond_manager @ BondsError::UserNotInMarket,
        has_one = claims @ BondsError::WrongClaimAccount,
        has_one = underlying_settlement @ BondsError::WrongUnderlyingSettlementAccount,
        has_one = ticket_settlement @ BondsError::WrongTicketSettlementAccount,
    )]
    pub margin_user: Account<'info, MarginUser>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = underlying_token_vault @ BondsError::WrongVault,
        has_one = bond_ticket_mint @ BondsError::WrongOracle,
        has_one = claims_mint @ BondsError::WrongClaimMint,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// SPL token program
    pub token_program: Program<'info, Token>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub claims: Account<'info, TokenAccount>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: token program checks it
    #[account(mut)]
    pub claims_mint: UncheckedAccount<'info>,

    /// CHECK: token program checks it
    pub underlying_token_vault: AccountInfo<'info>,
    /// CHECK: token program checks it
    pub bond_ticket_mint: AccountInfo<'info>,
    /// CHECK: token program checks it
    pub underlying_settlement: AccountInfo<'info>,
    /// CHECK: token program checks it
    pub ticket_settlement: AccountInfo<'info>,
}

pub fn handler(ctx: Context<Settle>) -> Result<()> {
    let claim_balance = ctx.accounts.claims.amount;
    let debt = &mut ctx.accounts.margin_user.debt;
    let total = debt.total();

    if claim_balance > total {
        burn!(ctx, claims_mint, claims, claim_balance - total)?;
    }
    if claim_balance < total {
        mint_to!(ctx, claims_mint, claims, total - claim_balance)?;
    }

    let assets = &mut ctx.accounts.margin_user.assets;
    mint_to!(
        ctx,
        bond_ticket_mint,
        ticket_settlement,
        assets.entitled_tickets
    )?;
    withdraw!(
        ctx,
        underlying_token_vault,
        underlying_settlement,
        assets.entitled_tokens
    )?;
    assets.entitled_tickets = 0;
    assets.entitled_tokens = 0;

    Ok(())
}

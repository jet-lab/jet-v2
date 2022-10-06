use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_margin::{AdapterResult, PositionChange};

use crate::{
    control::state::BondManager,
    margin::{
        events::MarginUserInitialized,
        state::{return_to_margin, MarginUser, MARGIN_USER_VERSION},
    },
    seeds,
    utils::init,
    BondsError,
};

#[derive(Accounts)]
pub struct InitializeMarginUser<'info> {
    /// The account tracking information related to this particular user
    #[account(
        init,
        seeds = [
            seeds::MARGIN_BORROWER,
            bond_manager.key().as_ref(),
            margin_account.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<MarginUser>(),
    )]
    pub borrower_account: Box<Account<'info, MarginUser>>,

    /// The signing authority for this user account
    #[account(
        constraint = margin_account.owner == &jet_margin::ID,
    )]
    pub margin_account: Signer<'info>,

    /// The Boheader account
    #[account(
        has_one = claims_mint @ BondsError::WrongClaimMint,
        has_one = collateral_mint @ BondsError::WrongCollateralMint
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// Token account used by the margin program to track the debt
    /// that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            borrower_account.key().as_ref(),
        ],
        bump,
        token::mint = claims_mint,
        token::authority = bond_manager,
        payer = payer)]
    pub claims: Box<Account<'info, TokenAccount>>,
    pub claims_mint: Box<Account<'info, Mint>>,

    /// Token account used by the margin program to track owned assets
    #[account(init,
        seeds = [
            seeds::COLLATERAL_NOTES,
            borrower_account.key().as_ref(),
        ],
        bump,
        token::mint = collateral_mint,
        token::authority = bond_manager,
        payer = payer)]
    pub collateral: Box<Account<'info, TokenAccount>>,
    pub collateral_mint: Box<Account<'info, Mint>>,

    pub underlying_settlement: Box<Account<'info, TokenAccount>>,
    pub ticket_settlement: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    /// Token metadata account needed by the margin program to register the claim position
    pub claims_metadata: AccountInfo<'info>,
}

pub fn handler(ctx: Context<InitializeMarginUser>) -> Result<()> {
    let user = &mut ctx.accounts.borrower_account;

    require_eq!(
        mint(&ctx.accounts.underlying_settlement.to_account_info())?,
        ctx.accounts.bond_manager.load()?.underlying_token_mint,
        BondsError::WrongUnderlyingTokenMint
    );
    require_eq!(
        mint(&ctx.accounts.ticket_settlement.to_account_info())?,
        ctx.accounts.bond_manager.load()?.bond_ticket_mint,
        BondsError::WrongTicketMint
    );

    init! {
        user = MarginUser {
            version: MARGIN_USER_VERSION,
            margin_account: ctx.accounts.margin_account.key(),
            bond_manager: ctx.accounts.bond_manager.key(),
            claims: ctx.accounts.claims.key(),
            collateral: ctx.accounts.collateral.key(),
            underlying_settlement: ctx.accounts.underlying_settlement.key(),
            ticket_settlement: ctx.accounts.ticket_settlement.key(),
        } ignoring {
            debt,
            assets,
        }
    }

    emit!(MarginUserInitialized {
        bond_manager: ctx.accounts.bond_manager.key(),
        borrower_account: ctx.accounts.borrower_account.key(),
        margin_account: ctx.accounts.margin_account.key(),
        underlying_settlement: ctx.accounts.underlying_settlement.key(),
        ticket_settlement: ctx.accounts.ticket_settlement.key(),
    });

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![(
                ctx.accounts.claims_mint.key(),
                vec![PositionChange::Register(ctx.accounts.claims.key())],
            )],
        },
    )
}

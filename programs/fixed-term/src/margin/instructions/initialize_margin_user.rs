use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_margin::{AdapterResult, PositionChange};

use crate::{
    control::state::Market,
    margin::{
        events::MarginUserInitialized,
        state::{return_to_margin, MarginUser, MARGIN_USER_VERSION},
    },
    seeds,
    utils::init,
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct InitializeMarginUser<'info> {
    /// The account tracking information related to this particular user
    #[account(
        init,
        seeds = [
            seeds::MARGIN_USER,
            market.key().as_ref(),
            margin_account.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<MarginUser>(),
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The signing authority for this user account
    #[account(
        constraint = margin_account.owner == &jet_margin::ID,
    )]
    pub margin_account: Signer<'info>,

    /// The fixed-term header account
    #[account(
        has_one = claims_mint @ FixedTermErrorCode::WrongClaimMint,
        has_one = ticket_collateral_mint @ FixedTermErrorCode::WrongCollateralMint
    )]
    pub market: AccountLoader<'info, Market>,

    /// Token account used by the margin program to track the debt
    /// that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            margin_user.key().as_ref(),
        ],
        bump,
        token::mint = claims_mint,
        token::authority = market,
        payer = payer)]
    pub claims: Box<Account<'info, TokenAccount>>,
    pub claims_mint: Box<Account<'info, Mint>>,

    /// Token account used by the margin program to track owned assets
    #[account(init,
        seeds = [
            seeds::TICKET_COLLATERAL_NOTES,
            margin_user.key().as_ref(),
        ],
        bump,
        token::mint = ticket_collateral_mint,
        token::authority = market,
        payer = payer)]
    pub ticket_collateral: Box<Account<'info, TokenAccount>>,
    pub ticket_collateral_mint: Box<Account<'info, Mint>>,

    /// Token account used by the margin program to track owned assets
    #[account(init,
        seeds = [
            seeds::TOKEN_COLLATERAL_NOTES,
            margin_user.key().as_ref(),
        ],
        bump,
        token::mint = token_collateral_mint,
        token::authority = market,
        payer = payer)]
    pub token_collateral: Box<Account<'info, TokenAccount>>,
    pub token_collateral_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    /// Token metadata account needed by the margin program to register the claim position
    pub claims_metadata: AccountInfo<'info>,

    /// Token metadata account needed by the margin program to register the collateral position
    pub ticket_collateral_metadata: AccountInfo<'info>,
}

pub fn handler(ctx: Context<InitializeMarginUser>) -> Result<()> {
    let user = &mut ctx.accounts.margin_user;

    init! {
        user = MarginUser {
            version: MARGIN_USER_VERSION,
            margin_account: ctx.accounts.margin_account.key(),
            market: ctx.accounts.market.key(),
            claims: ctx.accounts.claims.key(),
            ticket_collateral: ctx.accounts.ticket_collateral.key(),
            token_collateral: ctx.accounts.token_collateral.key(),
            borrow_roll_config: Default::default(),
            lend_roll_config: Default::default(),
        } ignoring {
            debt,
            assets,
        }
    }

    emit!(MarginUserInitialized {
        market: ctx.accounts.market.key(),
        margin_user: ctx.accounts.margin_user.key(),
        margin_account: ctx.accounts.margin_account.key(),
    });

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![
                (
                    ctx.accounts.claims_mint.key(),
                    vec![PositionChange::Register(ctx.accounts.claims.key())],
                ),
                (
                    ctx.accounts.ticket_collateral_mint.key(),
                    vec![PositionChange::Register(
                        ctx.accounts.ticket_collateral.key(),
                    )],
                ),
            ],
        },
    )
}

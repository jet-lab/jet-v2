use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{Token, TokenAccount},
};
use jet_margin::{AdapterResult, MarginAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    margin::state::{return_to_margin, MarginUser},
    market_token_manager::MarketTokenManager,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct Settle<'info> {
    /// The account tracking information related to this particular user
    #[account(mut,
        has_one = market @ FixedTermErrorCode::UserNotInMarket,
        has_one = claims @ FixedTermErrorCode::WrongClaimAccount,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Account<'info, MarginUser>,

    /// use accounting_invoke
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The `Market` account tracks global information related to this particular fixed term market
    #[account(
        has_one = underlying_token_vault @ FixedTermErrorCode::WrongVault,
        has_one = ticket_mint @ FixedTermErrorCode::WrongOracle,
        has_one = claims_mint @ FixedTermErrorCode::WrongClaimMint,
        has_one = ticket_collateral_mint @ FixedTermErrorCode::WrongCollateralMint,
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
    pub ticket_collateral: Account<'info, TokenAccount>,

    /// CHECK: token program checks it
    #[account(mut)]
    pub ticket_collateral_mint: UncheckedAccount<'info>,

    /// CHECK: token program checks it
    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,
    /// CHECK: token program checks it
    #[account(mut)]
    pub ticket_mint: AccountInfo<'info>,

    /// Where to receive owed tokens
    #[account(mut, address = get_associated_token_address(
        &margin_user.margin_account,
        &market.load().unwrap().underlying_token_mint,
    ))]
    pub underlying_settlement: AccountInfo<'info>,

    /// Where to receive owed tickets
    #[account(mut, address = get_associated_token_address(
        &margin_user.margin_account,
        &ticket_mint.key(),
    ))]
    pub ticket_settlement: AccountInfo<'info>,
}

pub fn handler(ctx: Context<Settle>) -> Result<()> {
    let claim_balance = ctx.accounts.claims.amount;
    let ctokens_held = ctx.accounts.ticket_collateral.amount;
    let assets = &ctx.accounts.margin_user.assets;
    let debt = ctx.accounts.margin_user.debt.total();
    let ctokens_deserved = assets.collateral()?;

    // Notify margin of the current debt owed to fixed-term market
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
            &ctx.accounts.ticket_collateral_mint,
            &ctx.accounts.ticket_collateral,
            ctokens_held - ctokens_deserved,
        )?;
    }
    if ctokens_held < ctokens_deserved {
        ctx.mint(
            &ctx.accounts.ticket_collateral_mint,
            &ctx.accounts.ticket_collateral,
            ctokens_deserved - ctokens_held,
        )?;
    }

    // Disburse entitled funds due to fills
    if assets.entitled_tickets > 0 {
        verify_settlement_account_registration(
            &*ctx.accounts.margin_account.load()?,
            ctx.accounts.ticket_mint.key(),
            ctx.accounts.ticket_settlement.key(),
            FixedTermErrorCode::TicketSettlementAccountNotRegistered,
        )?;
        ctx.mint(
            &ctx.accounts.ticket_mint,
            &ctx.accounts.ticket_settlement,
            assets.entitled_tickets,
        )?;
    }
    if assets.entitled_tokens > 0 {
        verify_settlement_account_registration(
            &*ctx.accounts.margin_account.load()?,
            ctx.accounts.market.load()?.underlying_token_mint.key(),
            ctx.accounts.underlying_settlement.key(),
            FixedTermErrorCode::UnderlyingSettlementAccountNotRegistered,
        )?;
        ctx.withdraw(
            &ctx.accounts.underlying_token_vault,
            &ctx.accounts.underlying_settlement,
            assets.entitled_tokens,
        )?;
    }

    // Update margin user assets to reflect the settlement
    ctx.accounts.margin_user.assets.entitled_tickets = 0;
    ctx.accounts.margin_user.assets.entitled_tokens = 0;

    ctx.accounts.margin_user.emit_all_balances();

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![],
        },
    )
}

fn verify_settlement_account_registration(
    margin_account: &MarginAccount,
    mint: Pubkey,
    token_account: Pubkey,
    error: FixedTermErrorCode,
) -> Result<()> {
    match margin_account.get_position(&mint) {
        Some(pos) => {
            if pos.address != token_account {
                msg!("The token account registered as a position ({:?}) for this mint ({mint:?}) does not match the settlement account ({token_account:?}).", pos.address);
                Err(error.into())
            } else {
                Ok(())
            }
        }
        None => {
            msg!("No position registered for this mint ({mint:?}), expected {token_account:?} to be registered.");
            Err(error.into())
        }
    }
}

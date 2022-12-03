use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::{
    AdapterPositionFlags, AdapterResult, PositionChange, PriceChangeInfo, MAX_ORACLE_STALENESS,
};

use crate::{
    control::{events::PositionRefreshed, state::Market},
    margin::state::{return_to_margin, MarginUser},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct RefreshPosition<'info> {
    /// The account tracking information related to this particular user
    #[account(
        has_one = market @ FixedTermErrorCode::UserNotInMarket,
        has_one = margin_account @ FixedTermErrorCode::WrongClaimAccount,
    )]
    pub margin_user: Account<'info, MarginUser>,

    /// CHECK: has_one on orderbook user
    pub margin_account: AccountInfo<'info>,

    /// The `Market` account tracks global information related to this particular fixed term market
    #[account(
        has_one = underlying_oracle @ FixedTermErrorCode::WrongOracle,
        has_one = ticket_oracle @ FixedTermErrorCode::WrongOracle,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The pyth price account
    /// CHECK: has_one on market
    pub underlying_oracle: AccountInfo<'info>,
    pub ticket_oracle: AccountInfo<'info>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RefreshPosition>, expect_price: bool) -> Result<()> {
    let market = ctx.accounts.market.load()?;
    let mut claim_changes = vec![PositionChange::Flags(
        AdapterPositionFlags::PAST_DUE,
        ctx.accounts.margin_user.debt.is_past_due(),
    )];
    let mut collateral_changes = vec![];
    let mut ticket_changes = vec![];

    // always try to update the price, but conditionally permit position updates if price fails
    // so we can continue to mark positions as past due even if there is an oracle failure
    match load_price(&ctx.accounts.underlying_oracle) {
        Ok(price) => claim_changes.push(price),
        Err(e) if expect_price => Err(e)?,
        Err(e) => msg!("skipping underlying price update due to error: {:?}", e),
    }
    match load_price(&ctx.accounts.ticket_oracle) {
        Ok(price) => {
            collateral_changes.push(price.clone());
            ticket_changes.push(price);
        }
        Err(e) if expect_price => Err(e)?,
        Err(e) => msg!("skipping ticket price update due to error: {:?}", e),
    }

    emit!(PositionRefreshed {
        margin_user: ctx.accounts.margin_user.key(),
    });

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![
                (market.claims_mint, claim_changes),
                (market.ticket_collateral_mint, collateral_changes),
                (market.ticket_mint, ticket_changes),
            ],
        },
    )
}

fn load_price(oracle_info: &AccountInfo) -> Result<PositionChange> {
    let oracle = pyth_sdk_solana::load_price_feed_from_account_info(oracle_info).map_err(|e| {
        msg!("oracle error in account {}: {:?}", oracle_info.key, e);
        error!(FixedTermErrorCode::OracleError)
    })?;
    // Required post pyth-sdk 0.6.1.
    // See https://github.com/pyth-network/pyth-sdk-rs/commit/4f4f8c79efcee6402a94dd81a0aa1750a1a12297
    let clock = Clock::get()?;
    let max_staleness = MAX_ORACLE_STALENESS as u64;
    let price = oracle
        .get_price_no_older_than(clock.unix_timestamp, max_staleness)
        .ok_or(FixedTermErrorCode::PriceMissing)?;
    let ema_price = oracle
        .get_ema_price_no_older_than(clock.unix_timestamp, max_staleness)
        .ok_or(FixedTermErrorCode::PriceMissing)?;
    Ok(PositionChange::Price(PriceChangeInfo {
        publish_time: price.publish_time,
        exponent: price.expo,
        value: price.price,
        confidence: price.conf,
        twap: ema_price.price,
    }))
}

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use jet_margin::{
    AdapterPositionFlags, AdapterResult, PositionChange, PriceChangeInfo, MAX_ORACLE_STALENESS,
};

use crate::{
    control::{events::PositionRefreshed, state::BondManager},
    margin::state::{return_to_margin, MarginUser},
    BondsError,
};

#[derive(Accounts)]
pub struct RefreshPosition<'info> {
    /// The account tracking information related to this particular user
    #[account(
        has_one = bond_manager @ BondsError::UserNotInMarket,
        has_one = margin_account @ BondsError::WrongClaimAccount,
    )]
    pub margin_user: Account<'info, MarginUser>,

    /// CHECK: has_one on orderbook user
    pub margin_account: AccountInfo<'info>,

    pub claims_mint: Account<'info, Mint>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = claims_mint @ BondsError::WrongClaimMint,
        has_one = underlying_oracle @ BondsError::WrongOracle,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The pyth price account
    /// CHECK: has_one on bond manager
    pub underlying_oracle: AccountInfo<'info>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RefreshPosition>, expect_price: bool) -> Result<()> {
    let mut position_changes = vec![PositionChange::Flags(
        AdapterPositionFlags::PAST_DUE,
        ctx.accounts.margin_user.debt.is_past_due(),
    )];

    // always try to update the price, but conditionally permit position updates if price fails
    // so we can continue to mark positions as past due even if there is an oracle failure
    match load_price(&ctx.accounts.underlying_oracle) {
        Ok(price) => position_changes.push(price),
        Err(e) if expect_price => Err(e)?,
        Err(e) => msg!("skipping price update due to error: {:?}", e),
    }

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![(ctx.accounts.claims_mint.key(), position_changes)],
        },
    )?;

    emit!(PositionRefreshed {
        borrower_account: ctx.accounts.margin_user.key(),
    });

    Ok(())
}

fn load_price(oracle_info: &AccountInfo) -> Result<PositionChange> {
    let oracle = pyth_sdk_solana::load_price_feed_from_account_info(oracle_info).map_err(|e| {
        msg!("oracle error: {:?}", e);
        error!(BondsError::OracleError)
    })?;
    // Required post pyth-sdk 0.6.1.
    // See https://github.com/pyth-network/pyth-sdk-rs/commit/4f4f8c79efcee6402a94dd81a0aa1750a1a12297
    let clock = Clock::get()?;
    let max_staleness = MAX_ORACLE_STALENESS as u64;
    let price = oracle
        .get_price_no_older_than(clock.unix_timestamp, max_staleness)
        .ok_or(BondsError::PriceMissing)?;
    let ema_price = oracle
        .get_ema_price_no_older_than(clock.unix_timestamp, max_staleness)
        .ok_or(BondsError::PriceMissing)?;
    Ok(PositionChange::Price(PriceChangeInfo {
        publish_time: price.publish_time,
        exponent: price.expo,
        value: price.price,
        confidence: price.conf,
        twap: ema_price.price,
    }))
}

use std::convert::TryInto;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::{AdapterPositionFlags, AdapterResult, PositionChange};
use pyth_sdk_solana::PriceFeed;

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
    let adapter_result = refresh_positions_deserialized(
        RefreshPositionsDeserialized {
            market: &*ctx.accounts.market.load()?,
            ticket_oracle: load_price(&ctx.accounts.ticket_oracle),
            underlying_oracle: load_price(&ctx.accounts.underlying_oracle),
            margin_user: &ctx.accounts.margin_user,
        },
        expect_price,
    )?;

    emit!(PositionRefreshed {
        margin_user: ctx.accounts.margin_user.key(),
    });

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &adapter_result,
    )
}

/// Exposes a simpler interface to the refresh positions logic. This enables
/// other runtimes to execute program behavior without needing to duplicate the
/// logic. Specifically, this is needed for the liquidator.
pub struct RefreshPositionsDeserialized<'a> {
    pub market: &'a Market,
    pub ticket_oracle: Result<PriceFeed>,
    pub underlying_oracle: Result<PriceFeed>,
    pub margin_user: &'a MarginUser,
}
pub fn refresh_positions_deserialized(
    accounts: RefreshPositionsDeserialized,
    expect_price: bool,
) -> Result<AdapterResult> {
    let market = accounts.market;
    let mut claim_changes = vec![PositionChange::Flags(
        AdapterPositionFlags::PAST_DUE,
        accounts.margin_user.debt.is_past_due(),
    )];
    let mut collateral_changes = vec![];

    // always try to update the price, but conditionally permit position updates if price fails
    // so we can continue to mark positions as past due even if there is an oracle failure
    match accounts.underlying_oracle {
        Ok(price) => claim_changes.push(PositionChange::Price(price.try_into()?)),
        Err(e) if expect_price => Err(e)?,
        Err(e) => msg!("skipping underlying price update due to error: {:?}", e),
    }
    match accounts.ticket_oracle {
        Ok(price) => collateral_changes.push(PositionChange::Price(price.try_into()?)),
        Err(e) if expect_price => Err(e)?,
        Err(e) => msg!("skipping ticket price update due to error: {:?}", e),
    }

    Ok(AdapterResult {
        position_changes: vec![
            (market.claims_mint, claim_changes),
            (market.ticket_collateral_mint, collateral_changes),
        ],
    })
}

fn load_price(oracle_info: &AccountInfo) -> Result<PriceFeed> {
    pyth_sdk_solana::load_price_feed_from_account_info(oracle_info).map_err(|e| {
        msg!("oracle error in account {}: {:?}", oracle_info.key, e);
        error!(FixedTermErrorCode::OracleError)
    })
}

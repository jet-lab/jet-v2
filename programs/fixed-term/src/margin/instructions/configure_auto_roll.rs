use anchor_lang::prelude::*;
use jet_margin::MarginAccount;
use jet_program_common::Fp32;

use crate::{
    control::state::Market,
    margin::state::{AutoRollConfig, BorrowAutoRollConfig, LendAutoRollConfig, MarginUser},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct ConfigureAutoRoll<'info> {
    /// The `MarginUser` account.
    /// This account is specific to a particular fixed-term market
    #[account(
        mut,
        has_one = margin_account,
        has_one = market,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The signing authority for this user account
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The fixed-term market this user belongs to
    pub market: AccountLoader<'info, Market>,
}

/// asserts that a limit price is set to a reasonable value
fn assert_limit_price(limit_price: u64) -> Result<()> {
    if limit_price >= Fp32::ONE.downcast_u64().unwrap() || limit_price == 0 {
        msg!(
            "Config price setting is invalid. Given price: [{}]",
            limit_price
        );
        return err!(FixedTermErrorCode::InvalidAutoRollConfig);
    }
    Ok(())
}

/// assert the new settings make sense
fn check_lend_config(config: &LendAutoRollConfig) -> Result<()> {
    assert_limit_price(config.limit_price)
}

/// assert the new settings make sense
fn check_borrow_config(config: &BorrowAutoRollConfig, market_tenor: u64) -> Result<()> {
    assert_limit_price(config.limit_price)?;
    if config.roll_tenor >= market_tenor || config.roll_tenor == 0 {
        msg!(
            "Config 'roll-tenor' is invalid, must be between the values of 0 and {}. Given value: {}.",
            market_tenor,
            config.roll_tenor
        );
        return err!(FixedTermErrorCode::InvalidAutoRollConfig);
    }
    Ok(())
}

pub fn handler(ctx: Context<ConfigureAutoRoll>, config: AutoRollConfig) -> Result<()> {
    let user = &mut ctx.accounts.margin_user;

    match config {
        AutoRollConfig::Borrow(config) => {
            check_borrow_config(&config, ctx.accounts.market.load()?.borrow_tenor)?;
            user.borrow_roll_config = Some(config);
        }
        AutoRollConfig::Lend(config) => {
            check_lend_config(&config)?;
            user.lend_roll_config = Some(config);
        }
    }

    Ok(())
}

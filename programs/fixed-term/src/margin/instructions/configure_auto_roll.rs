use anchor_lang::prelude::*;
use jet_margin::MarginAccount;
use jet_program_common::Fp32;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use crate::{
    margin::state::{AutoRollConfig, MarginUser},
    FixedTermErrorCode,
};

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
#[repr(u8)]
pub enum MarketSide {
    Borrowing,
    Lending,
}

#[derive(Accounts)]
pub struct ConfigureAutoRoll<'info> {
    /// The `MarginUser` account.
    /// This account is specific to a particular fixed-term market
    #[account(
        mut,
        has_one = margin_account,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The signing authority for this user account
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,
}

/// assert the new settings make sense
fn check_config(config: &AutoRollConfig) -> Result<()> {
    if config.limit_price >= Fp32::ONE.downcast_u64().unwrap() || config.limit_price == 0 {
        msg!(
            "Config price setting is invalid. Given price: [{}]",
            config.limit_price
        );
        return err!(FixedTermErrorCode::InvalidAutoRollConfig);
    }
    Ok(())
}

pub fn handler(ctx: Context<ConfigureAutoRoll>, side: u8, config: AutoRollConfig) -> Result<()> {
    check_config(&config)?;

    let user = &mut ctx.accounts.margin_user;
    match MarketSide::from_u8(side).unwrap() {
        MarketSide::Borrowing => user.borrow_roll_config = config,
        MarketSide::Lending => user.lend_roll_config = config,
    }
    Ok(())
}

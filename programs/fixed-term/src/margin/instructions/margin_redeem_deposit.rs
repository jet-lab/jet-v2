use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, tickets::instructions::redeem_deposit::*, FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginRedeemDeposit<'info> {
    #[account(mut,
        address = inner.owner.key(),
		constraint = margin_user.margin_account == inner.authority.key() @ FixedTermErrorCode::WrongMarginUserAuthority,
        has_one = ticket_collateral,
	)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut, address = inner.market.load()?.ticket_collateral_mint)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market]
    #[token_program]
    pub inner: RedeemDeposit<'info>,
}

impl<'info> MarginRedeemDeposit<'info> {
    pub fn redeem(&mut self) -> Result<()> {
        let redeemed = self.inner.redeem()?;
        self.margin_user
            .assets
            .redeem_deposit(self.inner.deposit.sequence_number, redeemed)?;

        anchor_spl::token::burn(
            CpiContext::new(
                self.inner.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.ticket_collateral_mint.to_account_info(),
                    from: self.ticket_collateral.to_account_info(),
                    authority: self.inner.market.to_account_info(),
                },
            )
            .with_signer(&[&self.inner.market.load()?.authority_seeds()]),
            redeemed,
        )?;

        self.margin_user.emit_asset_balances();

        Ok(())
    }
}

pub fn handler(ctx: Context<MarginRedeemDeposit>) -> Result<()> {
    ctx.accounts.redeem()
}

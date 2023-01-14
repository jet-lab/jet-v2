use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser,
    tickets::{
        instructions::redeem_deposit::*,
        state::{margin_redeem, MarginRedeemDepositAccounts, RedeemDepositAccounts},
    },
    FixedTermErrorCode,
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

pub fn handler(ctx: Context<MarginRedeemDeposit>) -> Result<()> {
    let accs = ctx.accounts;
    let accounts = &mut MarginRedeemDepositAccounts {
        margin_user: accs.margin_user.clone(),
        ticket_collateral: &accs.ticket_collateral,
        ticket_collateral_mint: &accs.ticket_collateral_mint,
        inner: &RedeemDepositAccounts {
            deposit: &accs.inner.deposit,
            owner: &accs.inner.owner,
            authority: &accs.inner.authority,
            payer: &accs.inner.payer,
            token_account: &accs.inner.token_account,
            market: &accs.inner.market,
            underlying_token_vault: &accs.inner.underlying_token_vault,
            token_program: &accs.inner.token_program,
        },
    };
    margin_redeem(accounts, true)
}

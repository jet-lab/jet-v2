use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser,
    orderbook::{
        instructions::lend_order::*,
        state::{LendOrderAccounts, MarginLendAccounts, OrderParams},
    },
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        constraint = margin_user.margin_account.key() == inner.authority.key(),
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market(orderbook_mut)]
    #[token_program]
    pub inner: LendOrder<'info>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams) -> Result<()> {
    let a = ctx.accounts;
    let accounts = &mut MarginLendAccounts {
        margin_user: a.margin_user.clone(),
        ticket_collateral: &a.ticket_collateral,
        ticket_collateral_mint: &a.ticket_collateral_mint,
        inner: &LendOrderAccounts {
            authority: &a.inner.authority,
            orderbook_mut: &a.inner.orderbook_mut,
            ticket_settlement: &a.inner.ticket_settlement,
            lender_tokens: &a.inner.lender_tokens,
            underlying_token_vault: &a.inner.underlying_token_vault,
            ticket_mint: &a.inner.ticket_mint,
            payer: &a.inner.payer,
            system_program: &a.inner.system_program,
            token_program: &a.inner.token_program,
        },
        adapter: ctx
            .remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    };
    accounts.margin_lend_order(&params, true)
}

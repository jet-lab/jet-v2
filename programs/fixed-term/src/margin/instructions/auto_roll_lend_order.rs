use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, orderbook::state::*, serialization::RemainingAccounts,
    tickets::state::TermDeposit,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollLendOrder<'info> {
    /// The off chain service authorized to roll this lend order
    pub roll_servicer: Signer<'info>,

    pub deposit: Account<'info, TermDeposit>,

    pub margin_user: Account<'info, MarginUser>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AutoRollLendOrder>) -> Result<()> {
    let lend_params = OrderParams {
        max_ticket_qty: u64::MAX,
        max_underlying_token_qty: ctx.accounts.deposit.amount,
        limit_price: ctx.accounts.margin_user.lend_roll_config.limit_price,
        match_limit: u64::MAX,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: true,
    };

    let (info, summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.deposit.owner,
        Side::Bid,
        lend_params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;
    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::{MarginUser, TermLoan},
    orderbook::state::*,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollBorrowOrder<'info> {
    /// The `MarginUser` account for this market
    #[account(
        mut,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::WrongMarket,
        has_one = margin_account @ FixedTermErrorCode::WrongMarginAccount,
	)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The `MarginAccount` this `TermDeposit` belongs to
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The `TermDeposit` account to roll
    #[account(
        mut,
        has_one = margin_user @ FixedTermErrorCode::WrongMarginUser,
        constraint = loan.payer == rent_receiver.key() @ FixedTermErrorCode::WrongRentReceiver,
    )]
    pub loan: Box<Account<'info, TermLoan>>,

    /// In the case the order matches, the new `TermLoan` to account for
    #[account(mut)]
    pub new_loan: AccountInfo<'info>,

    /// Reciever for rent from the closing of the TermDeposit
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,

    /// The accounts needed to interact with the orderbook
    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// Payer for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(_ctx: Context<AutoRollBorrowOrder>) -> Result<()> {
    Ok(())
}

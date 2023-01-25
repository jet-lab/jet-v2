use std::convert::TryFrom;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::{CrankAuthorization, Market},
    margin::state::{MarginUser, TermLoan},
    serialization::{AnchorAccount, Mut},
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct ConsumeEvents<'info> {
    /// The `Market` account tracks global information related to this particular fixed term market
    #[account(
        has_one = ticket_mint @ FixedTermErrorCode::WrongTicketMint,
        has_one = underlying_token_vault @ FixedTermErrorCode::WrongVault,
        has_one = orderbook_market_state @ FixedTermErrorCode::WrongMarketState,
        has_one = event_queue @ FixedTermErrorCode::WrongEventQueue,
    )]
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
    /// The ticket mint
    /// CHECK: has_one
    #[account(mut)]
    pub ticket_mint: AccountInfo<'info>,
    /// The market token vault
    /// CHECK: has_one
    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,

    // aaob accounts
    /// CHECK: handled by aaob
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    #[account(
        has_one = crank @ FixedTermErrorCode::WrongCrankAuthority,
        constraint = crank_authorization.airspace == market.load()?.airspace @ FixedTermErrorCode::WrongAirspaceAuthorization,
        constraint = crank_authorization.market == market.key() @ FixedTermErrorCode::WrongCrankAuthority,
    )]
    pub crank_authorization: Account<'info, CrankAuthorization>,
    pub crank: Signer<'info>,

    /// The account paying rent for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // remaining_accounts: [EventAccounts],
}

/// These are the additional accounts that need to be provided in the ix
/// for every event that will be processed.
/// For a fill, 2-6 accounts need to be appended to remaining_accounts
/// For an out, 1 account needs to be appended to remaining_accounts
#[allow(clippy::large_enum_variant)]
pub enum EventAccounts<'info> {
    Fill(FillAccounts<'info>),
    Out(OutAccounts<'info>),
}

#[allow(clippy::large_enum_variant)]
pub enum FillAccounts<'info> {
    Margin(MarginFillAccounts<'info>),
    Signer(FillAccount<'info>),
}

pub struct MarginFillAccounts<'info> {
    pub margin_user: AnchorAccount<'info, MarginUser, Mut>,
    pub term_account: Option<TermAccount<'info>>,
}

pub enum FillAccount<'info> {
    Token(AccountInfo<'info>),
    TermDeposit(AnchorAccount<'info, TermDeposit, Mut>),
}

impl<'info> FillAccount<'info> {
    pub fn as_token_account(&self) -> AccountInfo<'info> {
        match self {
            FillAccount::Token(info) => info.to_account_info(),
            _ => panic!(),
        }
    }
}

pub enum TermAccount<'info> {
    /// Use if AUTO_STAKE is set in the maker's callback
    Deposit(AnchorAccount<'info, TermDeposit, Mut>), // (ticket, user/owner)
    /// Use if NEW_DEBT is set in the maker's callback
    Loan(AnchorAccount<'info, TermLoan, Mut>), // (term loan, user)
}

impl<'info> TermAccount<'info> {
    pub fn term_deposit(&mut self) -> Result<&mut AnchorAccount<'info, TermDeposit, Mut>> {
        match self {
            TermAccount::Deposit(term_deposit) => Ok(term_deposit),
            _ => panic!(),
        }
    }

    pub fn term_loan(self) -> Result<AnchorAccount<'info, TermLoan, Mut>> {
        match self {
            TermAccount::Loan(term_loan) => Ok(term_loan),
            _ => panic!(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum OutAccounts<'info> {
    Margin(AnchorAccount<'info, MarginUser, Mut>),
    Signer(AccountInfo<'info>),
}

pub struct UserAccount<'info>(AccountInfo<'info>);
impl<'info> UserAccount<'info> {
    pub fn new(account: AccountInfo<'info>) -> Self {
        Self(account)
    }

    /// token account that will receive a deposit of underlying or tickets
    pub fn as_token_account(self) -> AccountInfo<'info> {
        self.0
    }

    pub fn margin_user(self) -> Result<AnchorAccount<'info, MarginUser, Mut>> {
        AnchorAccount::try_from(self.0)
    }

    pub fn pubkey(&self) -> Pubkey {
        self.0.key()
    }
}

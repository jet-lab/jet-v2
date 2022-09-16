use std::convert::TryInto;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_metadata::CrankMetadata;

use crate::{
    control::state::BondManager,
    margin::state::{MarginUser, Obligation},
    orderbook::state::EventQueue,
    serialization::{AnchorAccount, Mut},
    tickets::state::SplitTicket,
    BondsError,
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = bond_ticket_mint @ BondsError::WrongTicketMint,
        has_one = underlying_token_vault @ BondsError::WrongVault,
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = event_queue @ BondsError::WrongEventQueue,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,
    /// The market ticket mint
    /// CHECK: has_one
    #[account(mut)]
    pub bond_ticket_mint: AccountInfo<'info>,
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

    #[account(has_one = crank_signer @ BondsError::WrongCrankAuthority)]
    pub crank_metadata: Account<'info, CrankMetadata>,
    pub crank_signer: Signer<'info>,

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
pub enum EventAccounts<'info> {
    Fill(Box<FillAccounts<'info>>),
    Out(Box<OutAccounts<'info>>),
}

pub struct FillAccounts<'info> {
    pub maker: UserAccount<'info>,
    /// include if AUTO_STAKE or NEW_DEBT in callback
    pub loan: Option<LoanAccount<'info>>,
    pub maker_adapter: Option<EventQueue<'info>>,
    pub taker_adapter: Option<EventQueue<'info>>,
}

pub enum LoanAccount<'info> {
    /// Use if AUTO_STAKE is set in the maker's callback
    AutoStake(AnchorAccount<'info, SplitTicket, Mut>), // (ticket, user/owner)
    /// Use if NEW_DEBT is set in the maker's callback
    NewDebt(AnchorAccount<'info, Obligation, Mut>), // (obligation, user)
}

impl<'info> LoanAccount<'info> {
    pub fn auto_stake(self) -> Result<AnchorAccount<'info, SplitTicket, Mut>> {
        match self {
            LoanAccount::AutoStake(split_ticket) => Ok(split_ticket),
            _ => panic!(),
        }
    }

    pub fn new_debt(self) -> Result<AnchorAccount<'info, Obligation, Mut>> {
        match self {
            LoanAccount::NewDebt(obligation) => Ok(obligation),
            _ => panic!(),
        }
    }
}

pub struct OutAccounts<'info> {
    pub user: UserAccount<'info>,
    pub user_adapter_account: Option<EventQueue<'info>>,
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

    /// arbitrary unchecked account that will be granted ownership of a split ticket
    pub fn as_owner(self) -> AccountInfo<'info> {
        self.0
    }

    pub fn margin_user(self) -> Result<AnchorAccount<'info, MarginUser, Mut>> {
        self.0.try_into()
    }
}

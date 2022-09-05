use anchor_lang::prelude::*;
use bonds_metadata::CrankMetadata;

use crate::{
    control::state::BondManager,
    orderbook::state::{
        debt::Obligation, event_queue::EventQueue, user::OrderbookUser, CallbackInfo,
    },
    serialization::{seeds, AnchorAccount, Mut},
    tickets::state::SplitTicket,
    BondsError,
};

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,
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
    pub maker: UserData<'info>,
    pub taker: UserData<'info>,
    /// Include if AUTO_STAKE is set in the callback
    pub auto_stake: Option<AnchorAccount<'info, SplitTicket, Mut>>,
    /// Include if NEW_DEBT is set in the callback
    pub new_debt: Option<AnchorAccount<'info, Obligation, Mut>>,
    /// Include if EVENT_ADAPTER is set in the borrower callback
    /// Deserialization and validation is performed by the adapter program
    pub borrower_adapter_account: Option<EventQueue<'info>>,
    /// Include if EVENT_ADAPTER is set in the borrower callback
    /// Deserialization and validation is performed by the adapter program
    pub lender_adapter_account: Option<EventQueue<'info>>,
}
seeds! {
    auto_stake[b"auto_stake", lender, seeds_parameter]
    new_debt[b"new_debt", borrower, seeds_parameter]
}

pub struct OutAccounts<'info> {
    pub user: UserData<'info>,
    pub user_adapter_account: Option<EventQueue<'info>>,
}

/// The account plus some metadata about their role in the order
pub struct UserData<'info> {
    /// This needs to be provided as an account in the instruction
    pub account: AnchorAccount<'info, OrderbookUser, Mut>,

    /// This is extracted from the event queue and does not need to be provided as its own account
    pub callback: CallbackInfo,
}

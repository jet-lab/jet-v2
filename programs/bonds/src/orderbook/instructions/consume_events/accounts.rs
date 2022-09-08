use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_bonds_metadata::CrankMetadata;

use crate::{
    control::state::BondManager,
    margin::state::{MarginUser, Obligation},
    orderbook::state::{CallbackInfo, EventQueue},
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
pub enum EventAccounts<'a, 'info> {
    Fill(Box<FillAccounts<'a, 'info>>),
    Out(Box<OutAccounts<'a, 'info>>),
}

pub struct FillAccounts<'a, 'info> {
    pub maker: UserData<'a, 'info>,
    pub taker: UserData<'a, 'info>,
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

pub struct OutAccounts<'a, 'info> {
    pub user: UserData<'a, 'info>,
    pub user_adapter_account: Option<EventQueue<'info>>,
}

pub struct UserData<'a, 'info> {
    /// wallet for sending matched tokens
    pub vault: &'a AccountInfo<'info>,
    /// The signer for the order or the address of the `MarginUser` account
    pub key: Pubkey,
    /// If this user implemented a margin borrow order
    pub borrower_account: Option<Account<'info, MarginUser>>,
    /// The order tag for this order
    pub info: CallbackInfo,
}

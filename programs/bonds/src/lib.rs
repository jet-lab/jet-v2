//! An orderbook-based fixed term bond market hosted on the Solana blockchain
//!
//! # Interaction
//!
//! To interact with the bond market, users will initialize a PDA called an [`MarginUser`](struct@crate::orderbook::state::user::MarginUser).
//!
//! After `MarginUser` intialization, to place an order you must deposit underlying tokens or bond tickets into your account.
//! This will allow you to use the [`PlaceOrder`](struct@crate::orderbook::instructions::place_order::PlaceOrder) instruction, which
//! utilizes the orderbook to match borrowers and lenders.
//!
//! ### Lending
//!
//! To lend tokens, a user will deposit the underlying token into their `MarginUser` account. Then, they may post
//! orders on the book using the [`PlaceOrder`](struct@crate::orderbook::instructions::place_order::PlaceOrder) with a given set of
//! [`OrderParams`](struct@crate::orderbook::state::OrderParams).
//!
//! For example, to lend `1_000_000` tokens at 15% interest in a given market, a lender would specify:
//! ```ignore
//! OrderParams {
//!     /// We want as many bond tickets as the book will give us
//!     max_bond_ticket_qty: u64::MAX,
//!     /// we are lending 1_000_000 tokens
//!     max_underlying_token_qty: 1_000_000,
//!     /// use the crate function to generate a limit price
//!     limit_price: limit_price_from_f32((1.0 / 1.15)),
//!     /// limit the number of matches to 100
//!     match_limit: 100,
//!     /// Do not fail transaction if order crosses the spread
//!     post_only: false,
//!     /// If order does not get filled immediately, post remainder to the book
//!     post_allowed: true,
//!     /// stake generated tickets automatically, creating `SplitTicket`s
//!     auto_stake: true,
//! }
//!```
//!
//! ### Borrowing
//!
//! For borrowing, a user has two options. They can buy bond tickets from some market, and deposit them into their
//! `MarginUser` account. Or, they may use the `jet-margin` program to place collateralized borrow orders.
//!
//! In the case of a collateralized order, an `Obligation` will be minted to track the debt. A user must repay or face liquidation
//! by the `jet-margin` program.
//!
//! Example borrow order, where a borrower wants no more than 10% interest to borrow 100_000_000 tokens
//! ```ignore
//! OrderParams {
//!     /// We want to pay no more than 10%
//!     max_bond_ticket_qty: 110_000_000,
//!     /// we only need to borrow 100_000_000 tokens
//!     max_underlying_token_qty: 100_000_000,
//!     /// use the crate function to generate a limit price
//!     limit_price: limit_price_from_f32((1.0 / 1.10)),
//!     /// limit the number of matches to 100
//!     match_limit: 100,
//!     /// Do not fail transaction if order crosses the spread
//!     post_only: false,
//!     /// If order does not get filled immediately, post remainder to the book
//!     post_allowed: true,
//!     /// borrowers do not stake tickets
//!     auto_stake: false,
//! }
//! ```
//!
//! # Orderbook matching engine
//!
//! To facilitate the pairing of lenders and borrowers, the program utilizes the `agnostic-orderbook` crate to create an
//! orderbook. This orderbook allows lenders and borrowers to post orders using underlying tokens, held bond tickets, or, by utilizing `jet-margin` accounts,
//! a collateralized borrow order in lieu of held funds.
//!
//! ### EventQueue operation and Adapters
//!
//! The orderbook works by matching posted orders and pushing events to an `EventQueue` to be consumed by a crank operating offchain by
//! sending transactions to Solana.
//!
//! Some users may want to subscribe to events generated by the orderbook matching. To do this, a user must register with
//! the program an `Adapter` through their `MarginUser` account using the [`RegisterAdapter`](struct@crate::orderbook::instructions::event_adapter::RegisterAdapter) instruction. This instruction
//! creates an `AdapterEventQueue` PDA to which all processed orders containing the `MarginUser` account will be pushed.
//!
//! Users are responsible for handling the consumption logic for their adapter. To clear events after processing, use the [`PopAdapterEvents`](struct@crate::orderbook::instructions::event_adapter::PopAdapterEvents) instruction.
//!
//! # Bond Tickets
//!
//! The program abstracts the concept of a bond into bond tickets. Bond tickets are fungible spl tokens
//! that must be staked to claim their underlying value. In order to create bond tickets, a user must either
//! place a lend order on the orderbook, or exchange the token underlying the bond market (in practice, almost never
//! will users do this, as it locks their tokens for at least the tenor of the market).
//!
//! ### Ticket kinds and redemption
//!
//! The program allots for two types of ticket redemption. The [`ClaimTicket`](struct@crate::tickets::state::ClaimTicket) is given when
//! a user stakes directly with the program. As there is no information about the creation of the tickets, a `ClaimTicket` does not
//! have accounting for principal or interest, and only contains a redemptive value.
//!
//! Conversely, a [`SplitTicket`](struct@crate::tickets::state::SplitTicket) contains split principal and interest values. As well as
//! the slot it was minted. To create a `SplitTicket`, you must configure your [`OrderParams`](struct@crate::orderbook::state::OrderParams) `auto_stake` flag to
//! `true`. This will allow to program to immediately stake your tickets as the match event is processed.
//!
//! After the bond market tenor has passed, the ticket may be redeemed for the underlying value with the program. Also included are instructions
//! for transferring ownership of a ticket.
//!
//! # Debt and Obligations
//!
//! When using a `jet-margin` account to post a collateralized borrow order, an [`Obligation`](struct@crate::orderbook::state::debt::Obligation) is created to track
//! amounts owed to the program. `Obligation`s are either repaid manually by the user, or handled by an off-chain liquidator.

/// Program instructions and structs related to authoritative control of the program state
pub mod control;
/// Program instructions, methods and structs related to the use of margin accounts with the bonds program
pub mod margin;
/// Program instructions and structs related to use of the on chain orderbook
pub mod orderbook;
/// Program instructions and structs related to the redeemable bond tickets
pub mod tickets;

mod errors;
pub mod events;
pub use errors::BondsError;

/// Utilities for safely serializing and deserializing solana accounts
pub(crate) mod serialization;
/// local utilities for the crate
pub(crate) mod utils;

pub(crate) mod instructions;
use instructions::*;

#[macro_use]
extern crate bitflags;

use anchor_lang::prelude::*;
use orderbook::state::OrderParams;

declare_id!("JBond79m9K6HqYwngCjiJHb311GTXggo46kGcT2GijUc");

#[program]
pub mod jet_bonds {
    use super::*;

    //
    // Control Instructions
    // =============================================
    //

    /// authorize an address to run orderbook consume_event instructions
    pub fn authorize_crank(ctx: Context<AuthorizeCrank>) -> Result<()> {
        instructions::authorize_crank::handler(ctx)
    }

    /// unauthorize an address to run orderbook consume_event instructions
    pub fn revoke_crank(ctx: Context<RevokeCrank>) -> Result<()> {
        instructions::revoke_crank::handler(ctx)
    }

    /// Initializes a BondManager for a bond ticket market
    pub fn initialize_bond_manager(
        ctx: Context<InitializeBondManager>,
        params: InitializeBondManagerParams,
    ) -> Result<()> {
        instructions::initialize_bond_manager::handler(ctx, params)
    }

    /// Initializes a new orderbook
    pub fn initialize_orderbook(
        ctx: Context<InitializeOrderbook>,
        params: InitializeOrderbookParams,
    ) -> Result<()> {
        instructions::initialize_orderbook::handler(ctx, params)
    }

    /// Modify a `BondManager` account
    /// Authority use only
    pub fn modify_bond_manager(
        ctx: Context<ModifyBondManager>,
        data: Vec<u8>,
        offset: usize,
    ) -> Result<()> {
        instructions::modify_bond_manager::handler(ctx, data, offset)
    }

    /// Pause matching of orders placed in the orderbook
    pub fn pause_order_matching(ctx: Context<PauseOrderMatching>) -> Result<()> {
        instructions::pause_order_matching::handler(ctx)
    }

    /// Resume matching of orders placed in the orderbook
    /// NOTE: This instruction may have to be run several times to clear the
    /// existing matches. Check the `orderbook_market_state.pause_matching` variable
    /// to determine success
    pub fn resume_order_matching(ctx: Context<ResumeOrderMatching>) -> Result<()> {
        instructions::resume_order_matching::handler(ctx)
    }
    //
    // =============================================
    //

    //
    // Margin Instructions
    // =============================================
    //

    /// Create a new borrower account
    pub fn initialize_margin_user(ctx: Context<InitializeMarginUser>) -> Result<()> {
        instructions::initialize_margin_user::handler(ctx)
    }

    /// Place a borrow order by leveraging margin account value
    pub fn margin_borrow_order(
        ctx: Context<MarginBorrowOrder>,
        params: OrderParams,
        seed: Vec<u8>,
    ) -> Result<()> {
        instructions::margin_borrow_order::handler(ctx, params, seed)
    }

    /// Place a `Lend` order to the book by depositing tokens
    pub fn margin_lend_order(
        ctx: Context<MarginLendOrder>,
        params: OrderParams,
        seed: Vec<u8>,
    ) -> Result<()> {
        instructions::margin_lend_order::handler(ctx, params, seed)
    }

    /// Refresh the associated margin account `claims` for a given `MarginUser` account
    pub fn refresh_position(ctx: Context<RefreshPosition>, expect_price: bool) -> Result<()> {
        instructions::refresh_position::handler(ctx, expect_price)
    }

    /// Repay debt on an Obligation
    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        instructions::repay::handler(ctx, amount)
    }

    /// Settle payments to a margin account
    pub fn settle(ctx: Context<Settle>) -> Result<()> {
        instructions::settle::handler(ctx)
    }

    //
    // =============================================
    //

    //
    // Orderbook Instructions
    // =============================================
    //

    /// Place an order to the book to sell tickets, which will burn them
    pub fn sell_tickets_order(ctx: Context<SellTicketsOrder>, params: OrderParams) -> Result<()> {
        instructions::sell_tickets_order::handler(ctx, params)
    }

    /// Cancels an order on the book
    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
        instructions::cancel_order::handler(ctx, order_id)
    }

    /// Place a `Lend` order to the book by depositing tokens
    pub fn lend_order(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
        instructions::lend_order::handler(ctx, params, seed)
    }

    /// Crank specific instruction, processes the event queue
    pub fn consume_events<'a, 'b, 'info>(
        ctx: Context<'a, 'b, 'b, 'info, ConsumeEvents<'info>>,
        num_events: u32,
        seed_bytes: Vec<Vec<u8>>,
    ) -> Result<()> {
        instructions::consume_events::handler(ctx, num_events, seed_bytes)
    }

    //
    // =============================================
    //

    //
    // Ticket Instructions
    // =============================================
    //

    /// Exchange underlying token for bond tickets
    /// WARNING: tickets must be staked for redeption of underlying
    pub fn exchange_tokens(ctx: Context<ExchangeTokens>, amount: u64) -> Result<()> {
        instructions::exchange_tokens::handler(ctx, amount)
    }

    /// Redeems staked tickets for their underlying value
    pub fn redeem_ticket(ctx: Context<RedeemTicket>) -> Result<()> {
        instructions::redeem_ticket::handler(ctx)
    }

    /// Stakes bond tickets for later redemption
    pub fn stake_bond_tickets(
        ctx: Context<StakeBondTickets>,
        params: StakeBondTicketsParams,
    ) -> Result<()> {
        instructions::stake_bond_tickets::handler(ctx, params)
    }

    /// Transfer staked tickets to a new owner
    pub fn tranfer_ticket_ownership(
        ctx: Context<TransferTicketOwnership>,
        new_owner: Pubkey,
    ) -> Result<()> {
        instructions::transfer_ticket_ownership::handler(ctx, new_owner)
    }
    //
    // =============================================
    //

    //
    // Event Adapter Instructions
    // =============================================
    //

    /// Register a new EventAdapter for syncing to the orderbook events
    pub fn register_adapter(
        ctx: Context<RegisterAdapter>,
        params: RegisterAdapterParams,
    ) -> Result<()> {
        instructions::register_adapter::handler(ctx, params)
    }

    /// Pop the given number of events off the adapter queue
    /// Event logic is left to the outside program
    pub fn pop_adapter_events(ctx: Context<PopAdapterEvents>, num_events: u32) -> Result<()> {
        instructions::pop_adapter_events::handler(ctx, num_events)
    }
    //
    // =============================================
    //
}

pub mod seeds {
    use anchor_lang::prelude::constant;

    #[constant]
    pub const BOND_MANAGER: &[u8] = b"bond_manager";

    #[constant]
    pub const BOND_TICKET_ACCOUNT: &[u8] = b"bond_ticket_account";

    #[constant]
    pub const BOND_TICKET_MINT: &[u8] = b"bond_ticket_mint";

    #[constant]
    pub const CLAIM_TICKET: &[u8] = b"claim_ticket";

    #[constant]
    pub const CRANK_AUTHORIZATION: &[u8] = b"crank_authorization";

    #[constant]
    pub const CLAIM_NOTES: &[u8] = b"claim_notes";

    #[constant]
    pub const COLLATERAL_NOTES: &[u8] = b"collateral_notes";

    #[constant]
    pub const SPLIT_TICKET: &[u8] = b"split_ticket";

    #[constant]
    pub const EVENT_ADAPTER: &[u8] = b"event_adapter";

    #[constant]
    pub const OBLIGATION: &[u8] = b"obligation";

    #[constant]
    pub const ORDERBOOK_MARKET_STATE: &[u8] = b"orderbook_market_state";

    #[constant]
    pub const MARGIN_BORROWER: &[u8] = b"margin_borrower";

    #[constant]
    pub const UNDERLYING_TOKEN_VAULT: &[u8] = b"underlying_token_vault";
}

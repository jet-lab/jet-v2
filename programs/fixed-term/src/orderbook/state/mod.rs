mod event_queue;
mod lend;
pub use event_queue::*;
pub use lend::*;

use std::convert::TryInto;

use agnostic_orderbook::{
    instruction::{cancel_order, new_order},
    state::{
        critbit::Slab,
        critbit::{InnerNode, LeafNode, SlabHeader},
        event_queue::{EventQueueHeader, FillEvent},
        get_side_from_order_id, OrderSummary, SelfTradeBehavior, Side,
    },
};
use anchor_lang::{
    prelude::*,
    solana_program::{clock::UnixTimestamp, hash::hash},
};
use bytemuck::{Pod, Zeroable};

use crate::{
    control::state::Market, events::OrderCancelled, utils::orderbook_accounts, FixedTermErrorCode,
};

/// The tick_size used in fp32 operations on the orderbook
pub const TICK_SIZE: u64 = 1;

/// Find the len of the byteslab representing an orderbook side, given the maximum number of orders
pub const fn orderbook_slab_len(capacity: usize) -> usize {
    capacity * (LeafNode::LEN + CallbackInfo::LEN + InnerNode::LEN)
        + SlabHeader::LEN
        + LeafNode::LEN
        + CallbackInfo::LEN
        + 8
}

/// Calculated the length of the event queue buffer, given the maximum number of events
pub const fn event_queue_len(event_capacity: usize) -> usize {
    event_capacity * (FillEvent::LEN + 2 * CallbackInfo::LEN) + EventQueueHeader::LEN + 8
}

/// Set of accounts that are commonly needed together whenever the orderbook is modified
#[derive(Accounts, Clone)]
pub struct OrderbookMut<'info> {
    /// The `Market` account tracks global information related to this particular fixed term market
    #[account(
            mut,
            has_one = orderbook_market_state @ FixedTermErrorCode::WrongMarketState,
            has_one = bids @ FixedTermErrorCode::WrongBids,
            has_one = asks @ FixedTermErrorCode::WrongAsks,
            has_one = event_queue @ FixedTermErrorCode::WrongEventQueue,
            constraint = !market.load()?.orderbook_paused @ FixedTermErrorCode::OrderbookPaused,
        )]
    pub market: AccountLoader<'info, Market>,

    // aaob accounts
    /// CHECK: handled by aaob
    #[account(mut, owner = crate::ID @ FixedTermErrorCode::MarketStateNotProgramOwned)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub asks: AccountInfo<'info>,
}

impl<'info> OrderbookMut<'info> {
    pub fn underlying_mint(&self) -> Pubkey {
        self.market.load().unwrap().underlying_token_mint
    }

    pub fn ticket_mint(&self) -> Pubkey {
        self.market.load().unwrap().ticket_mint
    }

    pub fn vault(&self) -> Pubkey {
        self.market.load().unwrap().underlying_token_vault
    }

    pub fn ticket_collateral_mint(&self) -> Pubkey {
        self.market.load().unwrap().ticket_collateral_mint
    }

    pub fn claims_mint(&self) -> Pubkey {
        self.market.load().unwrap().claims_mint
    }

    fn place_order(
        &self,
        side: Side,
        params: OrderParams,
        info: &UserCallbackInfo,
    ) -> Result<SensibleOrderSummary> {
        let order_params = params.as_new_order_params(side, info.into());
        let limit_price = order_params.limit_price;
        let order_summary = new_order::process(
            &crate::id(),
            orderbook_accounts!(self, new_order),
            order_params,
        )?;
        require!(
            order_summary.posted_order_id.is_some() || order_summary.total_base_qty > 0,
            FixedTermErrorCode::OrderRejected
        );

        Ok(SensibleOrderSummary {
            summary: order_summary,
            limit_price,
        })
    }

    /// Place an order as a `MarginUser`
    pub fn place_margin_order(
        &mut self,
        side: Side,
        params: OrderParams,
        margin_account: Pubkey,
        margin_user: Pubkey,
        adapter: Option<Pubkey>,
        flags: CallbackFlags,
    ) -> Result<(MarginCallbackInfo, SensibleOrderSummary)> {
        let info = MarginCallbackInfo {
            order_tag: OrderTag::generate_from_market(&mut self.market, &margin_account)?,
            margin_account,
            margin_user,
            adapter_account_key: adapter.unwrap_or_default(),
            order_submitted: Clock::get()?.unix_timestamp,
            flags,
        };
        let summary = self.place_order(side, params, &UserCallbackInfo::Margin(info.clone()))?;
        Ok((info, summary))
    }

    /// Place an order as a generic signing authority
    #[allow(clippy::too_many_arguments)]
    pub fn place_signer_order(
        &mut self,
        side: Side,
        params: OrderParams,
        signer: Pubkey,
        ticket_account: Pubkey,
        token_account: Pubkey,
        adapter: Option<Pubkey>,
        flags: CallbackFlags,
    ) -> Result<(SignerCallbackInfo, SensibleOrderSummary)> {
        let info = SignerCallbackInfo {
            order_tag: OrderTag::generate_from_market(&mut self.market, &signer)?,
            signer,
            ticket_account,
            token_account,
            adapter_account_key: adapter.unwrap_or_default(),
            order_submitted: Clock::get()?.unix_timestamp,
            flags,
        };
        let summary = self.place_order(side, params, &UserCallbackInfo::Signer(info.clone()))?;
        Ok((info, summary))
    }

    /// cancels an order within the aaob
    /// you still need to act on the callback to reconcile any balances etc.
    pub fn cancel_order(
        &self,
        order_id: u128,
        owner: Pubkey,
    ) -> Result<(Side, CallbackFlags, OrderSummary)> {
        let side = get_side_from_order_id(order_id);
        let mut buf;
        let slab: Slab<CallbackInfo> = match side {
            Side::Bid => {
                buf = self.bids.data.borrow_mut();
                Slab::from_buffer(&mut buf, agnostic_orderbook::state::AccountTag::Bids)?
            }
            Side::Ask => {
                buf = self.asks.data.borrow_mut();
                Slab::from_buffer(&mut buf, agnostic_orderbook::state::AccountTag::Asks)?
            }
        };
        let handle = slab.find_by_key(order_id).ok_or_else(|| {
            msg!("Given Order ID: [{}]", order_id);
            error!(FixedTermErrorCode::OrderNotFound)
        })?;
        let info: UserCallbackInfo = (*slab.get_callback_info(handle)).into();

        let (info_owner, flags, order_tag) = match info.clone() {
            UserCallbackInfo::Margin(info) => {
                (info.margin_account, info.flags, info.order_tag.as_u128())
            }
            UserCallbackInfo::Signer(info) => (info.signer, info.flags, info.order_tag.as_u128()),
        };

        // drop the refs so the orderbook can borrow the slab data
        drop(buf);

        require_eq!(info_owner, owner, FixedTermErrorCode::WrongUserAccount);
        let orderbook_params = cancel_order::Params { order_id };
        let order_summary = agnostic_orderbook::instruction::cancel_order::process::<CallbackInfo>(
            &crate::id(),
            orderbook_accounts!(self, cancel_order),
            orderbook_params,
        )?;

        let eq_buf = &mut self.event_queue.data.borrow_mut();
        let mut event_queue =
            agnostic_orderbook::state::event_queue::EventQueue::<CallbackInfo>::from_buffer(
                eq_buf,
                agnostic_orderbook::state::AccountTag::EventQueue,
            )?;
        event_queue
            .push_back(
                agnostic_orderbook::state::event_queue::OutEvent {
                    tag: EventTag::Out as u8,
                    side: side as u8,
                    _padding: [0; 14],
                    order_id,
                    base_size: order_summary.total_base_qty,
                },
                Some(&CallbackInfo::from(&info)),
                None,
            )
            .map_err(|_| error!(FixedTermErrorCode::FailedToPushEvent))?;

        emit!(OrderCancelled {
            market: self.market.key(),
            authority: owner,
            order_tag,
        });

        Ok((side, flags, order_summary))
    }
}

#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Zeroable, Pod,
)]
#[repr(transparent)]
pub struct OrderTag(pub [u8; 16]);

impl OrderTag {
    /// Generates an order tag and mutates the market nonce
    pub fn generate_from_market(
        market_acc: &mut AccountLoader<Market>,
        user: &Pubkey,
    ) -> Result<Self> {
        let market = &mut market_acc.load_mut()?;
        let tag = Self::generate(market_acc.key().as_ref(), user.as_ref(), market.nonce);
        market.nonce = market.nonce.wrapping_add(1);

        Ok(tag)
    }
    //todo maybe this means we don't need owner to be stored in the CallbackInfo
    /// To generate an OrderTag, the program takes the sha256 hash of the orderbook user account
    /// and market pubkeys, a nonce tracked by the orderbook user account, and drops the
    /// last 16 bytes to create a 16-byte array
    fn generate(market_key_bytes: &[u8], user_key_bytes: &[u8], nonce: u64) -> Self {
        let nonce_bytes = bytemuck::bytes_of(&nonce);
        let bytes: &[u8] = &[market_key_bytes, user_key_bytes, nonce_bytes].concat();
        let hash: [u8; 32] = hash(bytes).to_bytes();
        let tag_bytes: &[u8; 16] = &hash[..16].try_into().unwrap();

        Self(*tag_bytes)
    }

    pub fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn as_u128(&self) -> u128 {
        u128::from_le_bytes(self.0)
    }
}

/// The CallbackInfo is information about an order that is stored in the Event Queue
/// used to manage order metadata
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
pub struct CallbackInfo {
    /// The order tag is generated by the program when submitting orders to the book
    /// Used to seed and track PDAs such as `TermLoan`
    order_tag: OrderTag,
    /// If the order was submit through the margin program, this is the MarginUser
    /// else, this is the signer who authorized token transfer
    /// used to determine ownership of resulting order or TermDeposit
    signer_or_margin_account: Pubkey,
    /// In the case of a generic signing user, this is the ticket account to mint
    /// towards on order fills
    ticket_account: Pubkey,
    /// margin user or token account to be deposited into on out
    /// the account that will be assigned ownership of any output resulting from
    /// an out. for margin orders this is the margin user. otherwise this is the
    /// token account to be deposited into.
    token_or_margin_user_account: Pubkey,
    /// Pubkey of the account that will receive the event information
    adapter_account_key: Pubkey,
    /// The unix timestamp for the slot that the order entered the aaob
    order_submitted: [u8; 8],
    /// configuration used by callback execution
    flags: CallbackFlags,
    _reserved: [u8; 14],
}

impl CallbackInfo {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub fn adapter(&self) -> Option<Pubkey> {
        let adapter = self.adapter_account_key;
        if adapter == Pubkey::default() {
            None
        } else {
            Some(adapter)
        }
    }

    pub fn order_submitted_timestamp(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.order_submitted)
    }

    pub fn from_signer_info(info: SignerCallbackInfo) -> Self {
        Self::from(&UserCallbackInfo::Signer(info))
    }

    pub fn from_margin_info(info: MarginCallbackInfo) -> Self {
        Self::from(&UserCallbackInfo::Margin(info))
    }

    pub fn owner(&self) -> Pubkey {
        self.signer_or_margin_account
    }

    pub fn flags(&self) -> CallbackFlags {
        self.flags
    }

    pub fn order_tag(&self) -> OrderTag {
        self.order_tag
    }
}

impl agnostic_orderbook::state::orderbook::CallbackInfo for CallbackInfo {
    type CallbackId = Pubkey;

    fn as_callback_id(&self) -> &Self::CallbackId {
        &self.signer_or_margin_account
    }
}

impl<'a, T: Into<&'a UserCallbackInfo>> From<T> for CallbackInfo {
    fn from(info: T) -> Self {
        match info.into() {
            UserCallbackInfo::Margin(info) => Self {
                order_tag: info.order_tag,
                signer_or_margin_account: info.margin_account,
                ticket_account: info.margin_user,
                token_or_margin_user_account: info.margin_user,
                adapter_account_key: info.adapter_account_key,
                order_submitted: info.order_submitted.to_le_bytes(),
                flags: info.flags,
                _reserved: [0u8; 14],
            },
            UserCallbackInfo::Signer(info) => Self {
                order_tag: info.order_tag,
                signer_or_margin_account: info.signer,
                ticket_account: info.ticket_account,
                token_or_margin_user_account: info.token_account,
                adapter_account_key: info.adapter_account_key,
                order_submitted: info.order_submitted.to_le_bytes(),
                flags: info.flags,
                _reserved: [0u8; 14],
            },
        }
    }
}

bitflags! {
    /// Binary flags for the `CallbackInfo`
    #[derive(Zeroable, Pod, Default)]
    #[repr(C)]
    pub struct CallbackFlags: u8 {
        /// any tickets purchased in this order should be automatically staked
        const AUTO_STAKE = 1;

        /// interest needs to start being accrued because this is new debt
        const NEW_DEBT   = 1 << 1;

        /// order placed by a MarginUser. margin user == fill_account == out_account
        const MARGIN     = 1 << 2;

        /// is this order subject to auto roll
        const AUTO_ROLL  = 1 << 3;
    }
}

/// CallbackInfo specific to the type of order
#[derive(Debug, Clone)]
pub enum UserCallbackInfo {
    /// The order was accounted by the MarginUser
    Margin(MarginCallbackInfo),
    /// The order was placed by a generic signing account
    Signer(SignerCallbackInfo),
}

impl UserCallbackInfo {
    /// Extracts the adapter pubkey
    pub fn adapter(&self) -> &Pubkey {
        match self {
            Self::Margin(info) => &info.adapter_account_key,
            Self::Signer(info) => &info.adapter_account_key,
        }
    }

    pub fn unwrap_margin(self) -> MarginCallbackInfo {
        match self {
            Self::Margin(info) => info,
            _ => panic!(),
        }
    }

    pub fn unwrap_signer(self) -> SignerCallbackInfo {
        match self {
            Self::Signer(info) => info,
            _ => panic!(),
        }
    }
}

impl<T: Into<CallbackInfo>> From<T> for UserCallbackInfo {
    fn from(info: T) -> Self {
        let info: CallbackInfo = info.into();
        if info.flags.contains(CallbackFlags::MARGIN) {
            UserCallbackInfo::Margin(info.into())
        } else {
            UserCallbackInfo::Signer(info.into())
        }
    }
}

/// CallbackInfo pertaining to an order placed through the margin instructions
#[derive(Debug, Clone)]
pub struct MarginCallbackInfo {
    /// The order tag is generated by the program when submitting orders to the book
    /// Used to seed and track PDAs such as `TermLoan`
    pub order_tag: OrderTag,
    /// The `MarginAccount` responsible for the order
    pub margin_account: Pubkey,
    /// The `MarginUser` account for the order
    pub margin_user: Pubkey,
    /// Pubkey of the account that will receive the event information
    pub adapter_account_key: Pubkey,
    /// The unix timestamp for the slot that the order entered the aaob
    pub order_submitted: UnixTimestamp,
    /// configuration used by callback execution
    pub flags: CallbackFlags,
}

impl<T: Into<CallbackInfo>> From<T> for MarginCallbackInfo {
    fn from(info: T) -> Self {
        let info: CallbackInfo = info.into();
        Self {
            order_tag: info.order_tag,
            margin_account: info.signer_or_margin_account,
            margin_user: info.token_or_margin_user_account,
            adapter_account_key: info.adapter_account_key,
            order_submitted: i64::from_le_bytes(info.order_submitted),
            flags: info.flags,
        }
    }
}

/// Callback information related to a generic signing account
#[derive(Debug, Clone)]
pub struct SignerCallbackInfo {
    /// The order tag is generated by the program when submitting orders to the book
    /// Used to seed and track PDAs such as `TermLoan`
    pub order_tag: OrderTag,
    /// The signing authority
    pub signer: Pubkey,
    /// The account to handle order fills if order is not set to make a `TermDeposit`
    pub ticket_account: Pubkey,
    /// The account to recompensate unused funds from orders leaving the book
    pub token_account: Pubkey,
    /// Pubkey of the account that will receive the event information
    pub adapter_account_key: Pubkey,
    /// The unix timestamp for the slot that the order entered the aaob
    pub order_submitted: UnixTimestamp,
    /// configuration used by callback execution
    pub flags: CallbackFlags,
}

impl<T: Into<CallbackInfo>> From<T> for SignerCallbackInfo {
    fn from(info: T) -> Self {
        let info: CallbackInfo = info.into();
        Self {
            order_tag: info.order_tag,
            signer: info.signer_or_margin_account,
            ticket_account: info.ticket_account,
            token_account: info.token_or_margin_user_account,
            adapter_account_key: info.adapter_account_key,
            order_submitted: i64::from_le_bytes(info.order_submitted),
            flags: info.flags,
        }
    }
}

/// Parameters needed for order placement
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Default, Clone, Copy)]
pub struct OrderParams {
    /// The maximum quantity of tickets to be traded.
    pub max_ticket_qty: u64,
    /// The maximum quantity of underlying token to be traded.
    pub max_underlying_token_qty: u64,
    /// The limit price of the order. This value is understood as a 32-bit fixed point number.
    pub limit_price: u64,
    /// The maximum number of orderbook postings to match in order to fulfill the order
    pub match_limit: u64,
    /// The order will not be matched against the orderbook and will be direcly written into it.
    ///
    /// The operation will fail if the order's limit_price crosses the spread.
    pub post_only: bool,
    /// Should the unfilled portion of the order be reposted to the orderbook
    pub post_allowed: bool,
    /// Should the purchased tickets be automatically staked with the ticket program
    pub auto_stake: bool,
    /// Should the resulting `TermLoan` or `TermDeposit` be subject to an auto roll
    pub auto_roll: bool,
}

// todo remove?
/// Trait to retrieve the posted quote values from an `OrderSummary`
pub trait WithQuoteQty {
    fn posted_quote(&self, price: u64) -> Result<u64>;
}

impl OrderParams {
    /// Transforms the locally defined struct into the expected struct for the agnostic orderbook
    pub fn as_new_order_params(
        &self,
        side: agnostic_orderbook::state::Side,
        callback_info: CallbackInfo,
    ) -> new_order::Params<CallbackInfo> {
        new_order::Params {
            max_base_qty: self.max_ticket_qty,
            max_quote_qty: self.max_underlying_token_qty,
            limit_price: self.limit_price,
            side,
            match_limit: self.match_limit,
            callback_info,
            post_only: self.post_only,
            post_allowed: self.post_allowed,
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
        }
    }
}

pub struct SensibleOrderSummary {
    limit_price: u64,
    summary: OrderSummary,
}

// fixme i think most of these are wrong. there may be issues with the aaob
// implementation which is a huge mess of spaghetti code
impl SensibleOrderSummary {
    pub fn summary(&self) -> OrderSummary {
        OrderSummary {
            posted_order_id: self.summary.posted_order_id,
            total_base_qty: self.summary.total_base_qty,
            total_quote_qty: self.summary.total_quote_qty,
            total_base_qty_posted: self.summary.total_base_qty_posted,
        }
    }

    // todo defensive rounding - depends on how this function is used
    pub fn quote_posted(&self) -> Result<u64> {
        fp32_mul(self.summary.total_base_qty_posted, self.limit_price)
            .ok_or_else(|| error!(FixedTermErrorCode::FixedPointDivision))
    }

    pub fn base_posted(&self) -> u64 {
        self.summary.total_base_qty_posted
    }

    pub fn quote_filled(&self) -> Result<u64> {
        Ok(self.summary.total_quote_qty - self.quote_posted()?)
    }

    pub fn base_filled(&self) -> u64 {
        self.summary.total_base_qty - self.summary.total_base_qty_posted
    }

    /// the total of all quote posted and filled
    /// NOT the same as the "max quote"
    pub fn quote_combined(&self) -> Result<u64> {
        // Ok(self.quote_posted()? + self.quote_filled())
        Ok(self.summary.total_quote_qty)
    }

    /// the total of all base posted and filled
    /// NOT the same as the "max base"
    pub fn base_combined(&self) -> u64 {
        // self.base_posted() + self.base_filled()
        self.summary.total_base_qty
    }
}

/// Multiply a `u64` with a fixed point 32 number
/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|e| safe_downcast(e >> 32))
}

/// a is fp0, b is fp32 and result is a/b fp0
pub fn fp32_div(a: u64, b_fp32: u64) -> Option<u64> {
    ((a as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(|x| x.try_into().ok())
}

fn safe_downcast(n: u128) -> Option<u64> {
    static BOUND: u128 = u64::MAX as u128;
    if n > BOUND {
        None
    } else {
        Some(n as u64)
    }
}

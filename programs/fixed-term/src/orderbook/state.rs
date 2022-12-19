use crate::{
    control::state::Market, events::OrderCancelled, utils::orderbook_accounts, FixedTermErrorCode,
};
use agnostic_orderbook::{
    instruction::cancel_order,
    state::{critbit::Slab, get_side_from_order_id, Side},
};
use agnostic_orderbook::{
    instruction::new_order,
    state::{
        critbit::{InnerNode, LeafNode, SlabHeader},
        event_queue::{EventQueueHeader, FillEvent, OutEvent},
        OrderSummary, SelfTradeBehavior,
    },
};
use anchor_lang::{
    prelude::*,
    solana_program::{clock::UnixTimestamp, hash::hash},
};
use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::TryInto;

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
#[derive(Accounts)]
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

    #[allow(clippy::too_many_arguments)]
    pub fn place_order(
        &self,
        owner: Pubkey,
        side: Side,
        params: OrderParams,
        fill: Pubkey,
        out: Pubkey,
        adapter: Option<Pubkey>,
        flags: CallbackFlags,
    ) -> Result<(CallbackInfo, SensibleOrderSummary)> {
        let mut manager = self.market.load_mut()?;
        let auto_roll = if params.auto_roll {
            CallbackFlags::AUTO_ROLL
        } else {
            CallbackFlags::default()
        };
        let callback_info = CallbackInfo::new(
            self.market.key(),
            owner,
            fill,
            out,
            adapter.unwrap_or_default(),
            Clock::get()?.unix_timestamp,
            flags | auto_roll,
            manager.nonce,
        );
        manager.nonce += 1;

        let order_params = params.as_new_order_params(side, callback_info);
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

        Ok((
            callback_info,
            SensibleOrderSummary {
                summary: order_summary,
                limit_price,
            },
        ))
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
        let info = *slab.get_callback_info(handle);

        let info_owner = info.owner;
        let flags = info.flags;
        let order_tag = info.order_tag.as_u128();

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
                Some(&info),
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
    //todo maybe this means we don't need owner to be stored in the CallbackInfo
    /// To generate an OrderTag, the program takes the sha256 hash of the orderbook user account
    /// and market pubkeys, a nonce tracked by the orderbook user account, and drops the
    /// last 16 bytes to create a 16-byte array
    pub fn generate(market_key_bytes: &[u8], user_key_bytes: &[u8], nonce: u64) -> OrderTag {
        let nonce_bytes = bytemuck::bytes_of(&nonce);
        let bytes: &[u8] = &[market_key_bytes, user_key_bytes, nonce_bytes].concat();
        let hash: [u8; 32] = hash(bytes).to_bytes();
        let tag_bytes: &[u8; 16] = &hash[..16].try_into().unwrap();

        OrderTag(*tag_bytes)
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
    pub order_tag: OrderTag,
    /// authority who signed to place the order and is permitted to cancel the
    /// order. for margin orders, this is the owner of the MarginUser, which is
    /// the margin account PDA from the margin program.
    pub owner: Pubkey,
    /// the account that will be assigned ownership of any output resulting from
    /// a fill. for margin orders this is the margin user. for auto-stake, this
    /// account will be set as the split ticket owner. otherwise this is the
    /// token account to be deposited into.
    pub fill_account: Pubkey,
    /// margin user or token account to be deposited into on out
    /// the account that will be assigned ownership of any output resulting from
    /// an out. for margin orders this is the margin user. otherwise this is the
    /// token account to be deposited into.
    pub out_account: Pubkey,
    /// Pubkey of the account that will receive the event information
    pub adapter_account_key: Pubkey,
    /// The unix timestamp for the slot that the order entered the aaob
    pub order_submitted: [u8; 8],
    /// configuration used by callback execution
    pub flags: CallbackFlags,
    _reserved: [u8; 14],
}

impl CallbackInfo {
    pub const LEN: usize = std::mem::size_of::<Self>();
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        market_key: Pubkey,
        owner: Pubkey,
        fill_account: Pubkey,
        out_account: Pubkey,
        adapter: Pubkey,
        order_submitted: UnixTimestamp,
        flags: CallbackFlags,
        nonce: u64,
    ) -> Self {
        let order_tag = OrderTag::generate(market_key.as_ref(), fill_account.as_ref(), nonce);
        Self {
            owner,
            fill_account,
            out_account,
            order_tag,
            adapter_account_key: adapter,
            order_submitted: order_submitted.to_le_bytes(),
            flags,
            _reserved: [0u8; 14],
        }
    }

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
}

impl agnostic_orderbook::state::orderbook::CallbackInfo for CallbackInfo {
    type CallbackId = Pubkey;

    fn as_callback_id(&self) -> &Self::CallbackId {
        &self.owner
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

/// Parameters needed for order placement
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
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

#[account(zero_copy)]
pub struct EventAdapterMetadata {
    /// Signing authority over this Adapter
    pub owner: Pubkey,
    /// The `Market` this adapter belongs to
    pub market: Pubkey,
    /// The `MarginUser` account this adapter is registered for
    pub orderbook_user: Pubkey,
}

impl EventAdapterMetadata {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub fn space(num_events: u32) -> usize {
        num_events as usize * (FillEvent::LEN + 2 * CallbackInfo::LEN)
            + Self::LEN
            + EventQueueHeader::LEN
            + 16 // anchor discriminator and agnostic-orderbook tag
    }
}

#[derive(FromPrimitive, Clone, Copy, CheckedBitPattern, NoUninit)]
#[repr(u8)]
pub(crate) enum EventTag {
    Fill,
    Out,
}

pub(crate) type GenericEvent = FillEvent;

pub trait Event {
    fn to_generic(&mut self) -> &GenericEvent;
}

impl Event for FillEvent {
    fn to_generic(&mut self) -> &GenericEvent {
        self.tag = EventTag::Fill as u8;
        self
    }
}

impl Event for OutEvent {
    fn to_generic(&mut self) -> &GenericEvent {
        self.tag = EventTag::Out as u8;
        bytemuck::cast_ref(self)
    }
}

pub enum OrderbookEvent {
    Fill(FillInfo),
    Out(OutInfo),
}

pub struct FillInfo {
    pub event: FillEvent,
    pub maker_info: CallbackInfo,
    pub taker_info: CallbackInfo,
}
pub struct OutInfo {
    pub event: OutEvent,
    pub info: CallbackInfo,
}

/// todo: algebraic type parameter could provide compile time checks on use
/// as market or user adapter
#[derive(Clone)]
pub struct EventQueue<'a> {
    info: AccountInfo<'a>,
    header: EventQueueHeader,
    capacity: usize,
    event_ptr: usize,
    callback_ptr: usize,
    is_adapter: bool,
}

impl<'a> Key for EventQueue<'a> {
    fn key(&self) -> Pubkey {
        self.info.key()
    }
}

impl<'a> EventQueue<'a> {
    const ADAPTER_OFFSET: usize = EventAdapterMetadata::LEN + 8;

    pub fn deserialize_market(info: AccountInfo<'a>) -> Result<Self> {
        require!(
            info.owner == &crate::id(),
            FixedTermErrorCode::WrongEventQueue
        );
        Self::deserialize(info, false)
    }

    pub fn deserialize_user_adapter(info: AccountInfo<'a>) -> Result<Self> {
        require!(
            info.owner != &crate::id(),
            FixedTermErrorCode::UserDoesNotOwnAdapter
        );
        Self::deserialize(info, true)
    }

    fn deserialize(info: AccountInfo<'a>, is_adapter: bool) -> Result<Self> {
        let adapter_offset = if is_adapter { Self::ADAPTER_OFFSET } else { 0 };
        let buf = &info.data.borrow();
        let capacity = (buf.len() - 8 - EventQueueHeader::LEN - adapter_offset)
            / (FillEvent::LEN + 2 * CallbackInfo::LEN);

        let header_ptr = 8 + adapter_offset;
        let event_ptr = header_ptr + EventQueueHeader::LEN;
        let callback_ptr = event_ptr + capacity * FillEvent::LEN;

        let header = EventQueueHeader::deserialize(&mut &buf[header_ptr..])?;

        Ok(Self {
            info: info.clone(),
            capacity,
            event_ptr,
            callback_ptr,
            header,
            is_adapter,
        })
    }

    /// Pushes the given event to the back of the queue
    pub fn push_event<E: Event>(
        &mut self,
        mut event: E,
        maker_callback_info: Option<&CallbackInfo>,
        taker_callback_info: Option<&CallbackInfo>,
    ) -> std::result::Result<(), Error> {
        let mut buf = self.info.data.borrow_mut();
        let generic_event = event.to_generic();
        let event_idx = (self.header.head as usize + self.header.count as usize) % self.capacity;

        let events: &mut [FillEvent] =
            bytemuck::cast_slice_mut(&mut buf[self.event_ptr..self.callback_ptr]);
        events[event_idx] = *generic_event;

        self.header.count += 1;

        let callback_infos: &mut [CallbackInfo] =
            bytemuck::cast_slice_mut(&mut buf[self.callback_ptr..]);
        if let Some(c) = maker_callback_info {
            callback_infos[event_idx * 2] = *c;
        }

        if let Some(c) = taker_callback_info {
            callback_infos[event_idx * 2 + 1] = *c;
        }

        Ok(())
    }

    /// Attempts to remove the number of events from the top of the queue
    pub fn pop_events(&mut self, num_events: u32) -> Result<()> {
        let capped_number_of_entries_to_pop = std::cmp::min(self.header.count, num_events as u64);
        self.header.count -= capped_number_of_entries_to_pop;
        self.header.head =
            (self.header.head + capped_number_of_entries_to_pop) % (self.capacity as u64);
        Ok(())
    }

    fn get_event(&self, event_idx: usize) -> OrderbookEvent {
        let buf = self.info.data.borrow();
        let events: &[FillEvent] = bytemuck::cast_slice(&buf[self.event_ptr..self.callback_ptr]);
        let callback: &[CallbackInfo] = bytemuck::cast_slice(&buf[self.callback_ptr..]);

        let event = &events[event_idx];
        match EventTag::from_u8(event.tag).unwrap() {
            EventTag::Fill => OrderbookEvent::Fill(FillInfo {
                event: *event,
                maker_info: callback[2 * event_idx],
                taker_info: callback[2 * event_idx + 1],
            }),
            EventTag::Out => OrderbookEvent::Out(OutInfo {
                event: *bytemuck::cast_ref(event),
                info: callback[2 * event_idx],
            }),
        }
    }

    pub fn iter(&self) -> QueueIterator<'a> {
        QueueIterator {
            queue: self.clone(),
            current_index: 0,
            remaining: self.header.count,
        }
    }
}

impl<'a> Drop for EventQueue<'a> {
    fn drop(&mut self) {
        let mut buf = self.info.data.borrow_mut();

        let offset = match self.is_adapter {
            true => Self::ADAPTER_OFFSET + 8,
            false => 8,
        };
        self.header
            .serialize(&mut (&mut buf[offset..] as &mut [u8]))
            .unwrap();
    }
}

/// Utility struct for iterating over a queue
pub struct QueueIterator<'a> {
    queue: EventQueue<'a>,
    current_index: usize,
    remaining: u64,
}

impl<'a> Iterator for QueueIterator<'a> {
    type Item = OrderbookEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let event_idx =
            (self.queue.header.head as usize + self.current_index) % self.queue.capacity;
        self.current_index += 1;
        self.remaining -= 1;
        Some(self.queue.get_event(event_idx))
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

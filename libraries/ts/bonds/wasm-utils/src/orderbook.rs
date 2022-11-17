use agnostic_orderbook::state::critbit::Slab;
use bytemuck::{Pod, Zeroable};
use jet_program_common::Fp32;
use ouroboros::self_referencing;

use crate::{
    console_log,
    methods::{base_to_quote, quote_to_base},
};

use super::{error::FixedTermWasmError, interest_pricing::f64_to_fp32};
use js_sys::{Array, Uint8Array};

use std::fmt::Display;

use wasm_bindgen::prelude::*;

use super::interest_pricing::fp32_to_f64;

pub type Result<T> = std::result::Result<T, JsError>;

// FIXME Import this from bonds program instead; perhaps move to common lib.
/// The CallbackInfo is information about an order that is stored in the Event Queue
/// used to manage order metadata
#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
pub struct CallbackInfo {
    /// The order tag is generated by the program when submitting orders to the book
    /// Used to seed and track PDAs such as `Obligation`
    pub order_tag: [u8; 16],
    /// authority permitted to modify the order
    pub owner: [u8; 32],
    /// margin user, split ticket owner, or token account to be deposited into on fill
    pub fill_account: [u8; 32],
    /// margin user or token account to be deposited into on out
    pub out_account: [u8; 32],
    /// Pubkey of the account that will recieve the event information
    pub adapter_account_key: [u8; 32],
    /// The unix timestamp for the slot that the order entered the aaob
    pub order_submitted: [u8; 8],
    /// configuration used by callback execution
    pub flags: u8,
    _reserved: [u8; 14],
}

/// Converts a buffer from an orderbook side into an array of orders on the book
///
/// Params:
///
/// `slab_bytes`: a `UInt8Array` from the AccountInfo data
#[wasm_bindgen]
pub fn get_orders_from_slab(slab_bytes: &[u8]) -> Array {
    let buf = &mut slab_bytes.to_owned();
    let buf_clone = &mut slab_bytes.to_owned();

    let slab: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf).unwrap();
    let slab_clone: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf_clone).unwrap();

    Array::from_iter(
        slab_clone
            .into_iter(true)
            .map(|leaf| {
                let handle = slab.find_by_key(leaf.key).unwrap();
                let callback = slab.get_callback_info(handle);
                Order {
                    owner: Uint8Array::from(&callback.owner[..]),
                    order_tag: Uint8Array::from(&callback.order_tag[..]),
                    base_size: leaf.base_quantity,
                    quote_size: Fp32::upcast_fp32(leaf.price())
                        .decimal_u64_mul(leaf.base_quantity)
                        .unwrap(),
                    limit_price: leaf.price(),
                    order_id: Uint8Array::from(&leaf.key.to_le_bytes()[..]),
                }
            })
            .map(JsValue::from),
    )
}

#[wasm_bindgen]
pub struct Order {
    /// Pukbey of the signer allowed to make changes to this order
    #[wasm_bindgen(getter_with_clone)]
    pub owner: Uint8Array,
    /// order tag used to track pdas related to this order
    /// 16 byte hash derived
    #[wasm_bindgen(getter_with_clone)]
    pub order_tag: Uint8Array,
    /// The orderId as found on the orderbook
    /// a u128, used for cancel order instructions
    #[wasm_bindgen(getter_with_clone)]
    pub order_id: Uint8Array,
    /// Total bond ticket worth of the order
    pub base_size: u64,
    /// Total underlying token worth of the order
    pub quote_size: u64,
    /// Fixed point 32 representation of the price
    pub limit_price: u64,
}

/// Represents a 3-tuple of order parameters, returned when calculating order parameters from a given
/// amount and interest rate
#[wasm_bindgen]
pub struct OrderAmount {
    /// max base quantity for an order
    pub base: u64,
    /// max quote quantity for an order
    pub quote: u64,
    /// fixed-point 32 limit price value
    pub price: u64,
}

/// Identifies the role of the user.
#[wasm_bindgen]
pub enum Actor {
    Lender,
    Borrower,
}

/// Actions that an `Actor` may be taking.
#[wasm_bindgen]
#[derive(PartialEq, Eq)]
pub enum Action {
    RequestLoan,
    RequestBorrow,
    LendNow,
    BorrowNow,
}

/// The estimated result of emitting an order into the orderbook, as produced
/// by `estimated_order_outcome`.
#[wasm_bindgen]
pub struct EstimatedOrderOutcome {
    /// The approxiamte volume-weighted average price achieved, FP32 representation.
    pub vwap: u64,
    /// The number of quote lamports filled.
    pub filled_quote: u64,
    /// The number of base lamports filled.
    pub filled_base: u64,
    /// The number of requested quote lamports left unfilled.
    pub unfilled_quote: u64,
    /// The number of matches contributing to the result.
    pub matches: u32,
}

/// Estimates the outcome of a hypothetical order.
///
/// quote_size is the amount of quote lamports the order seeks to trade.
/// taker is the would-be owner of the order.
/// order_type categories the would be order.
/// limit_price is required for limit order types; FP32 representation.
/// resting_orders is an array of orders that could be hit by the taker order. The "opposite"
///     side of the book from the taker, so to speak. These must be ordered appropriately
///     or an error will be returned, ie sorted ascending (descending) by limit price for
///     asks (bids).
#[wasm_bindgen]
pub fn estimate_order_outcome(
    quote_size: u64,
    taker: Uint8Array,
    order_type: Action,
    limit_price: Option<u64>,
    resting_orders: Array,
) -> Result<EstimatedOrderOutcome> {
    console_error_panic_hook::set_once();

    let mut unfilled_quote = quote_size;
    let mut filled_quote = 0_u64;
    let mut filled_base = 0_u64;
    let mut matches = 0_u32;
    let mut last_price = match order_type {
        // To enforce ordering of resting orders
        Action::RequestLoan | Action::LendNow => u64::MIN,
        Action::RequestBorrow | Action::BorrowNow => u64::MAX,
    };
    let limit_price = match limit_price {
        Some(p) => p,
        None => {
            if order_type == Action::RequestBorrow || order_type == Action::RequestLoan {
                return Err(FixedTermWasmError::LimitPriceRequired.into());
            } else {
                0_u64
            }
        }
    };

    console_log!(
        "Simulating matches with limit price {} and quote size: {}",
        fp32_to_f64(limit_price),
        quote_size,
    );
    console_log!("There are {} resting orders.", resting_orders.length());

    for item in resting_orders.iter() {
        if unfilled_quote == 0 {
            break;
        }

        let order: Order = item.clone().into();

        console_log!("Order {}: {}", matches, order);

        // Ensure that we're processing appropriately ordered resting_orders. The correct
        // ordering depends on whether the hypothetical order being processed is hitting
        // bids or asks.

        match order_type {
            Action::RequestLoan | Action::LendNow => {
                if order.limit_price < last_price {
                    return Err(FixedTermWasmError::RestingOrdersNotSorted.into());
                }
            }
            Action::RequestBorrow | Action::BorrowNow => {
                if order.limit_price > last_price {
                    return Err(FixedTermWasmError::RestingOrdersNotSorted.into());
                }
            }
        };
        last_price = order.limit_price;

        // For limit orders we might be done matching.

        if (order_type == Action::RequestLoan && order.limit_price > limit_price)
            || (order_type == Action::RequestBorrow && order.limit_price < limit_price)
        {
            break;
        }

        console_log!("limit price check okay");

        // Current behaviour of the debt markets is to abort the transaction on self-match,
        // so we indicate as much if we antipate a self-match. This check must be maintained
        // consitently with the actual orderbook configuration.

        if order.owned_by(&taker) {
            return Err(FixedTermWasmError::SelfMatch.into());
        }

        // Determine the filled amount, making sure to respect both base and quote
        // quantities available on the order.

        console_log!("* check unfilled_quote={} nonzero", unfilled_quote);
        console_log!(
            "* quote_to_base() gives: {}",
            quote_to_base(
                std::cmp::min(unfilled_quote, order.quote_size),
                order.limit_price,
            )
        );

        let bfill = std::cmp::min(
            quote_to_base(
                std::cmp::min(unfilled_quote, order.quote_size),
                order.limit_price,
            ),
            order.base_size,
        );
        let qfill = base_to_quote(bfill, order.limit_price);

        console_log!("Order filled {} quote and {} base", qfill, bfill);

        filled_quote += qfill;
        filled_base += bfill;
        unfilled_quote = unfilled_quote.saturating_sub(qfill);
        matches += 1;

        console_log!(
            "Cumulatively filled {} quote and {} base",
            filled_quote,
            filled_base
        );
        console_log!("{} quote remaining", unfilled_quote);
    }

    console_log!(
        "filled_quote: {}, filled_base: {}",
        filled_quote,
        filled_base
    );
    console_log!("vwap_f64: {}", filled_quote as f64 / filled_base as f64);
    console_log!(
        "vwap_fp32: {}",
        f64_to_fp32(filled_quote as f64 / filled_base as f64)
    );

    let vwap = if filled_quote == 0 || filled_base == 0 {
        0
    } else {
        f64_to_fp32(filled_quote as f64 / filled_base as f64)
    };

    Ok(EstimatedOrderOutcome {
        vwap,
        filled_quote,
        filled_base,
        unfilled_quote,
        matches,
    })
}

/// Adjusts a price to tolerate an amonut of slipapge.
///
/// price is the limit price of an order; FP32 representation.
/// fraction is the amount of slippage, eg 0.05 would be 5%.
/// trader is the party constructing the order.
///
/// Returns the adjusted price in FP32 representation.
#[wasm_bindgen]
pub fn with_slippage(price: u64, fraction: f64, actor: Actor) -> u64 {
    let scale = match actor {
        Actor::Lender => 1_f64 + fraction,
        Actor::Borrower => 1_f64 - fraction,
    };

    f64_to_fp32(fp32_to_f64(price) * scale)
}

#[wasm_bindgen(module = "/src/cast.js")]
extern "C" {
    // This will let Rust regain ownership of Order
    #[wasm_bindgen(js_name = castInto)]
    pub fn cast_into_order(value: JsValue) -> Order;
}

impl From<JsValue> for Order {
    fn from(order: JsValue) -> Self {
        cast_into_order(order)
    }
}

#[wasm_bindgen]
impl Order {
    #[wasm_bindgen(constructor)]
    pub fn new(
        owner: Uint8Array,
        order_tag: Uint8Array,
        order_id: Uint8Array,
        base_size: u64,
        quote_size: u64,
        limit_price: u64,
    ) -> Order {
        Order {
            owner,
            order_tag,
            order_id,
            base_size,
            quote_size,
            limit_price,
        }
    }

    pub fn owned_by(&self, candidate: &Uint8Array) -> bool {
        let mut a = [0; 32];
        let mut b = [0; 32];

        self.owner.copy_to(&mut a);
        candidate.copy_to(&mut b);

        a == b
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Order(quote_size={}, base_size={}, limit_price={})",
            self.quote_size,
            self.base_size,
            fp32_to_f64(self.limit_price),
        )
    }
}

// #[wasm_bindgen]
pub struct OrderbookFacade {
    state: OrderbookFacadeState,
}

#[self_referencing]
struct OrderbookFacadeState {
    bids_slab_buffer: Vec<u8>,
    #[borrows(mut bids_slab_buffer)]
    #[covariant]
    bids: Slab<'this, ()>,

    asks_slab_buffer: Vec<u8>,
    #[borrows(mut asks_slab_buffer)]
    #[covariant]
    asks: Slab<'this, ()>,
}

// #[wasm_bindgen(js_class=OrderbookFacade)]
impl OrderbookFacade {
    // #[wasm_bindgen(js_name=updateState)]
    pub fn update_state(&mut self, bids_buffer: &[u8], asks_buffer: &[u8]) -> Result<()> {
        self.state = OrderbookFacadeStateBuilder {
            bids_slab_buffer: bids_buffer.to_owned(),
            asks_slab_buffer: asks_buffer.to_owned(),
            bids_builder: |buf: &mut Vec<u8>| Slab::from_buffer_unchecked(buf).unwrap(), // FIXME Don't panic
            asks_builder: |buf: &mut Vec<u8>| Slab::from_buffer_unchecked(buf).unwrap(), // FIXME Don't panic
        }
        .build();

        Ok(())
    }

    // #[wasm_bindgen(js_name=viewLiquidity)]
    pub fn view_liquidity() -> Result<()> {
        todo!()
    }

    // #[wasm_bindgen(js_name=estimateFill)]
    pub fn estimate_fill() -> Result<()> {
        todo!()
    }
}

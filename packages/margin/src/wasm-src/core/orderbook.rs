use agnostic_orderbook::state::critbit::Slab;
use bonfida_utils::fp_math::{fp32_div, fp32_mul_ceil, fp32_mul_floor};
use jet_fixed_term::{
    margin::origination_fee::{borrow_order_qty, loan_to_disburse},
    orderbook::state::{CallbackInfo, OrderTag},
};
use jet_program_common::interest_pricing::{f64_to_fp32, fp32_to_f64};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_program::pubkey::Pubkey;

use crate::orderbook::methods::price_to_rate;

pub struct OrderbookModel {
    tenor: u64,
    origination_fee: u64,
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    /// Pukbey of the signer allowed to make changes to this order
    pub owner: Pubkey,
    /// Order tag used to track pdas related to this order
    pub order_tag: OrderTag,
    /// Total ticket worth of the order
    pub base_size: u64,
    /// Fixed point 32 representation of the price
    pub price: u64,
}

impl Order {
    fn matches(&self, action: Action, limit_price: u64) -> bool {
        match action {
            Action::Lend => self.price <= limit_price,
            Action::Borrow => self.price >= limit_price,
        }
    }

    fn precedes(&self, action: Action, limit_price: u64) -> bool {
        match action {
            Action::Lend => self.price >= limit_price,
            Action::Borrow => self.price <= limit_price,
        }
    }

    fn quote_size(&self, side: Side) -> Option<u64> {
        side.base_to_quote(self.base_size, self.price)
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[repr(C)]
pub enum Action {
    Lend,
    Borrow,
}

impl Action {
    pub fn worst_price(&self) -> u64 {
        match self {
            Action::Lend => 1 << 32,
            Action::Borrow => 1,
        }
    }

    fn side_posted(&self) -> Side {
        match self {
            Action::Lend => Side::LoanOffer,
            Action::Borrow => Side::LoanRequest,
        }
    }
}

impl From<&str> for Action {
    fn from(name: &str) -> Self {
        let name = name.to_lowercase();
        match name.as_str() {
            "lend" | "lendnow" | "offerloan" => Action::Lend,
            "borrow" | "borrownow" | "requestloan" => Action::Borrow,
            _ => panic!(), // TODO try_from
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Side {
    LoanRequest,
    LoanOffer,
}

impl Side {
    pub fn matching(action: Action) -> Self {
        match action {
            Action::Lend => Side::LoanRequest,
            Action::Borrow => Side::LoanOffer,
        }
    }

    pub fn base_to_quote(&self, base: u64, price: u64) -> Option<u64> {
        match self {
            Side::LoanOffer => fp32_mul_ceil(base, price),
            Side::LoanRequest => fp32_mul_floor(base, price),
        }
    }
}

impl From<&str> for Side {
    fn from(name: &str) -> Self {
        let name = name.to_lowercase();
        match name.as_str() {
            "asks" | "loanrequest" => Side::LoanRequest,
            "bids" | "loanoffer" => Side::LoanOffer,
            _ => panic!(), // TODO try_from
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiquiditySample {
    pub side: Side,
    pub total_quote_qty: u64,
    pub sample_quote_qty: u64,
    pub points: Vec<LiquidityObservation>,
}

#[derive(Serialize, Debug, Clone)]
pub struct LiquidityObservation {
    pub cumulative_base: u64,
    pub cumulative_quote: u64,
    pub cumulative_price: f64,
    pub cumulative_rate: f64,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct LiquiditySampleV2 {
    #[serde_as(as = "DisplayFromStr")]
    pub base: Pubkey,
    #[serde_as(as = "DisplayFromStr")]
    pub quote: Pubkey,
    /// (f64, u64) => (rate, cumulative_quote)
    pub bids: Vec<(f64, u64)>,
    /// (f64, u64) => (rate, cumulative_quote)
    pub asks: Vec<(f64, u64)>,
    pub price_range: [f64; 2],
    pub liquidity_range: [u64; 2],
}

#[derive(Deserialize)]
pub struct OrderbookSnapshot {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

const MIN_BASE_SIZE_POSTED: u64 = 10;

impl OrderbookModel {
    // TODO Add min base size posted as parameter
    pub fn new(tenor: u64, origination_fee: u64) -> Self {
        Self {
            tenor,
            origination_fee,
            bids: vec![],
            asks: vec![],
        }
    }

    pub fn refresh(&mut self, bids_buffer: &[u8], asks_buffer: &[u8]) {
        let extract_orders = |buffer: &[u8], ascending: bool| {
            let buf1 = &mut buffer.to_owned();
            let slab1: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf1).unwrap();

            let buf2 = &mut buffer.to_owned();
            let slab2: Slab<CallbackInfo> = Slab::from_buffer_unchecked(buf2).unwrap();

            slab2
                .into_iter(ascending)
                .map(|leaf| {
                    let handle = slab1.find_by_key(leaf.key).unwrap();
                    let callback = slab1.get_callback_info(handle);
                    Order {
                        owner: callback.owner(),
                        order_tag: callback.order_tag(),
                        base_size: leaf.base_quantity,
                        price: leaf.price(),
                    }
                })
                .collect()
        };

        self.bids = extract_orders(bids_buffer, false);
        self.asks = extract_orders(asks_buffer, true);
    }

    pub fn refresh_from_snapshot(&mut self, snapshot: OrderbookSnapshot) {
        self.bids = snapshot.bids;
        self.asks = snapshot.asks;
    }

    // TODO Interpolate on a set of points instead
    pub fn sample_liquidity(
        &self,
        side: Side,
        max_quote_qty: Option<u64>,
        append_zero_point: Option<bool>,
    ) -> LiquiditySample {
        let max_quote_qty = max_quote_qty.unwrap_or(u64::MAX);
        let append_zero_point = append_zero_point.unwrap_or(true);

        let mut total_base_qty = 0;
        let mut total_quote_qty = 0;
        let mut sample_quote_qty = 0;
        let mut points = vec![];

        for &Order {
            base_size,
            price: limit_price,
            ..
        } in self.orders_on(side)
        {
            if total_quote_qty >= max_quote_qty {
                break;
            }

            let quote_size = side.base_to_quote(base_size, limit_price).unwrap();
            total_base_qty += base_size;
            total_quote_qty += quote_size;
            let cumulative_price = total_quote_qty as f64 / total_base_qty as f64;
            let cumulative_rate =
                price_to_rate(f64_to_fp32(cumulative_price), self.tenor) as f64 / 10_000_f64; // price_to_rate returns bps;
            points.push(LiquidityObservation {
                cumulative_base: total_base_qty,
                cumulative_quote: total_quote_qty,
                cumulative_price,
                cumulative_rate,
            });

            sample_quote_qty += quote_size;
        }

        if append_zero_point && !points.is_empty() {
            let cumulative_base = 0;
            let cumulative_quote = 0;
            let cumulative_price = points[0].cumulative_price;
            let cumulative_rate = points[0].cumulative_rate;

            points.insert(
                0,
                LiquidityObservation {
                    cumulative_base,
                    cumulative_quote,
                    cumulative_price,
                    cumulative_rate,
                },
            );
        }

        LiquiditySample {
            side,
            total_quote_qty,
            sample_quote_qty,
            points,
        }
    }

    pub fn sample_liquidity_v2(&self, max_quote_qty: u64) -> LiquiditySampleV2 {
        let bid_liquidity =
            self.sample_liquidity(Side::LoanOffer, Some(max_quote_qty), Some(false));
        let ask_liquidity =
            self.sample_liquidity(Side::LoanRequest, Some(max_quote_qty), Some(false));

        let bids = bid_liquidity
            .points
            .iter()
            .map(|p| (p.cumulative_rate, p.cumulative_quote))
            .collect::<Vec<_>>();
        let asks = ask_liquidity
            .points
            .iter()
            .map(|p| (p.cumulative_rate, p.cumulative_quote))
            .rev()
            .collect::<Vec<_>>();

        LiquiditySampleV2 {
            base: Pubkey::default(),
            quote: Pubkey::default(),
            liquidity_range: [
                bids.first()
                    .map_or(0, |(_, qty)| *qty)
                    .max(asks.last().map_or(0, |(_, qty)| *qty)),
                bids.last()
                    .map_or(0, |(_, qty)| *qty)
                    .max(asks.first().map_or(0, |(_, qty)| *qty)),
            ],
            price_range: [
                asks.first().map_or(f64::NAN, |(rate, _)| *rate),
                bids.last().map_or(f64::NAN, |(rate, _)| *rate),
            ],
            bids,
            asks,
        }
    }

    pub fn would_match(&self, action: Action, limit_price: u64) -> bool {
        let orders = self.orders_on(Side::matching(action));

        if orders.is_empty() {
            false
        } else {
            orders[0].matches(action, limit_price)
        }
    }

    // TODO Don't panic
    pub fn simulate_taker(
        &self,
        action: Action,
        user_quote_qty: u64,
        limit_price: Option<u64>,
        user: Option<Pubkey>,
    ) -> TakerSimulation {
        let with_limit_price = limit_price.is_some();
        let limit_price = limit_price.unwrap_or_else(|| action.worst_price());
        let side = Side::matching(action);

        let fee_quote_qty = match action {
            Action::Lend => 0,
            Action::Borrow => {
                borrow_order_qty(user_quote_qty, self.origination_fee) - user_quote_qty
            }
        };
        let total_quote_qty = user_quote_qty + fee_quote_qty;

        let mut self_match = false;
        let mut filled_quote_qty = 0;
        let mut filled_user_qty = 0;
        let mut filled_fee_qty = 0;
        let mut filled_base_qty = 0;
        let mut unfilled_quote_qty = total_quote_qty;
        let mut fills = vec![];
        for order in self.orders_on(side) {
            if unfilled_quote_qty > 0 && order.matches(action, limit_price) {
                if let Some(user) = user {
                    if order.owner == user {
                        self_match = true;
                    }
                }

                let maker_base_qty = order.base_size;
                let unfilled_base_qty = fp32_div(unfilled_quote_qty, order.price).unwrap();
                let fill_base_qty = maker_base_qty.min(unfilled_base_qty);
                let fill_quote_qty = side.base_to_quote(fill_base_qty, order.price).unwrap();

                let fill_user_qty = loan_to_disburse(fill_quote_qty, self.origination_fee);
                let fill_fee_qty = fill_quote_qty - fill_user_qty;

                let fill = Fill {
                    base_qty: fill_base_qty,
                    quote_qty: fill_quote_qty,
                    user_qty: fill_user_qty,
                    fee_qty: fill_fee_qty,
                    price: fp32_to_f64(order.price),
                };

                fills.push(fill);

                filled_quote_qty += fill_quote_qty;
                filled_user_qty += fill_user_qty;
                filled_fee_qty += fill_fee_qty;
                filled_base_qty += fill_base_qty;
                unfilled_quote_qty -= fill_quote_qty;
            } else {
                break;
            }
        }

        let filled_vwap = if filled_base_qty > 0 {
            filled_quote_qty as f64 / filled_base_qty as f64
        } else {
            f64::NAN
        };
        let filled_vwar = if filled_vwap.is_normal() {
            price_to_rate(f64_to_fp32(filled_vwap), self.tenor) as f64 / 10_000_f64
        } else {
            f64::NAN
        };

        let limit_price = if with_limit_price {
            fp32_to_f64(limit_price)
        } else {
            f64::NAN
        };

        TakerSimulation {
            total_quote_qty,
            user_quote_qty,
            fee_quote_qty,
            limit_price,
            would_match: !fills.is_empty(),
            self_match,
            matches: fills.len(),
            filled_quote_qty,
            filled_user_qty,
            filled_fee_qty,
            filled_base_qty,
            filled_vwap,
            filled_vwar,
            fills,
            unfilled_quote_qty,
        }
    }

    pub fn simulate_maker(
        &self,
        action: Action,
        user_quote_qty: u64,
        limit_price: u64,
        user: Option<Pubkey>,
    ) -> MakerSimulation {
        let fill_sim = self.simulate_taker(action, user_quote_qty, Some(limit_price), user);

        let mut maker_sim = MakerSimulation {
            total_quote_qty: fill_sim.total_quote_qty,
            user_quote_qty: fill_sim.user_quote_qty,
            fee_quote_qty: fill_sim.fee_quote_qty,
            limit_price: fp32_to_f64(limit_price),
            full_quote_qty: 0,
            full_base_qty: 0,
            full_vwap: f64::NAN,
            full_vwar: f64::NAN,
            would_post: false,
            depth: 0,
            posted_quote_qty: 0,
            posted_user_qty: 0,
            posted_fee_qty: 0,
            posted_base_qty: 0,
            posted_vwap: f64::NAN,
            posted_vwar: f64::NAN,
            preceding_base_qty: 0,
            preceding_quote_qty: 0,
            preceding_vwap: f64::NAN,
            preceding_vwar: f64::NAN,
            would_match: false,
            self_match: false,
            matches: 0,
            filled_quote_qty: 0,
            filled_user_qty: 0,
            filled_fee_qty: 0,
            filled_base_qty: 0,
            filled_vwap: f64::NAN,
            filled_vwar: f64::NAN,
            fills: vec![],
        };

        if fill_sim.would_match {
            maker_sim.would_match = true;
            maker_sim.self_match = fill_sim.self_match;
            maker_sim.matches = fill_sim.matches;
            maker_sim.filled_quote_qty = fill_sim.filled_quote_qty;
            maker_sim.filled_user_qty = fill_sim.filled_user_qty;
            maker_sim.filled_fee_qty = fill_sim.filled_fee_qty;
            maker_sim.filled_base_qty = fill_sim.filled_base_qty;
            maker_sim.filled_vwap = fill_sim.filled_vwap;
            maker_sim.filled_vwar = fill_sim.filled_vwar;
            maker_sim.fills = fill_sim.fills;
        } else {
            maker_sim.filled_vwap = f64::NAN;
            maker_sim.filled_vwar = f64::NAN;
        }

        let remaining_base_qty = if fill_sim.would_match {
            fp32_div(fill_sim.unfilled_quote_qty, limit_price).unwrap()
        } else {
            fp32_div(maker_sim.total_quote_qty, limit_price).unwrap()
        };

        maker_sim.would_post = remaining_base_qty >= MIN_BASE_SIZE_POSTED;

        if maker_sim.would_post {
            let side = action.side_posted();
            let orders = self.orders_on(side);

            let mut depth: usize = 0;
            let mut preceding_base_qty = 0;
            let mut preceding_quote_qty = 0;
            for order in orders {
                if order.precedes(action, limit_price) {
                    depth += 1;
                    preceding_base_qty += order.base_size;
                    preceding_quote_qty += order.quote_size(side).unwrap(); // FIXME CHECK
                } else {
                    break;
                }
            }

            let preceding_vwap = if preceding_quote_qty > 0 {
                preceding_quote_qty as f64 / preceding_base_qty as f64
            } else {
                f64::NAN
            };
            let preceding_vwar = if preceding_vwap.is_normal() {
                price_to_rate(f64_to_fp32(preceding_vwap), self.tenor) as f64 / 10_000_f64
            } else {
                f64::NAN
            };

            let posted_base_qty = remaining_base_qty;
            let posted_quote_qty = side.base_to_quote(remaining_base_qty, limit_price).unwrap();

            let posted_vwap = if posted_quote_qty > 0 {
                posted_quote_qty as f64 / posted_base_qty as f64
            } else {
                f64::NAN
            };
            let posted_vwar = if posted_vwap.is_normal() {
                price_to_rate(f64_to_fp32(posted_vwap), self.tenor) as f64 / 10_000_f64
            } else {
                f64::NAN
            };

            let posted_user_qty = loan_to_disburse(posted_quote_qty, self.origination_fee);
            let posted_fee_qty = posted_quote_qty - posted_user_qty;

            maker_sim.depth = depth;
            maker_sim.posted_quote_qty = posted_quote_qty;
            maker_sim.posted_user_qty = posted_user_qty;
            maker_sim.posted_fee_qty = posted_fee_qty;
            maker_sim.posted_base_qty = posted_base_qty;
            maker_sim.posted_vwap = posted_vwap;
            maker_sim.posted_vwar = posted_vwar;
            maker_sim.preceding_quote_qty = preceding_quote_qty;
            maker_sim.preceding_base_qty = preceding_base_qty;
            maker_sim.preceding_vwap = preceding_vwap;
            maker_sim.preceding_vwar = preceding_vwar;
        } else {
            maker_sim.posted_vwap = f64::NAN;
            maker_sim.posted_vwar = f64::NAN;
            maker_sim.preceding_vwap = f64::NAN;
            maker_sim.preceding_vwar = f64::NAN;
        }

        let full_quote_qty = maker_sim.filled_quote_qty + maker_sim.posted_quote_qty;
        let full_base_qty = maker_sim.filled_base_qty + maker_sim.posted_base_qty;

        let full_vwap = if full_quote_qty > 0 {
            full_quote_qty as f64 / full_base_qty as f64
        } else {
            f64::NAN
        };
        let full_vwar = if full_vwap.is_normal() {
            price_to_rate(f64_to_fp32(full_vwap), self.tenor) as f64 / 10_000_f64
        } else {
            f64::NAN
        };

        maker_sim.full_quote_qty = full_quote_qty;
        maker_sim.full_base_qty = full_base_qty;
        maker_sim.full_vwap = full_vwap;
        maker_sim.full_vwar = full_vwar;

        maker_sim
    }

    pub fn loan_offers(&self) -> &Vec<Order> {
        &self.bids
    }

    pub fn loan_requests(&self) -> &Vec<Order> {
        &self.asks
    }

    fn orders_on(&self, side: Side) -> &Vec<Order> {
        match side {
            Side::LoanOffer => &self.bids,
            Side::LoanRequest => &self.asks,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TakerSimulation {
    pub total_quote_qty: u64,
    pub user_quote_qty: u64,
    pub fee_quote_qty: u64,
    pub limit_price: f64,

    pub would_match: bool,
    pub self_match: bool,
    pub matches: usize,
    pub filled_quote_qty: u64,
    pub filled_user_qty: u64,
    pub filled_fee_qty: u64,
    pub filled_base_qty: u64,
    pub filled_vwap: f64,
    pub filled_vwar: f64,
    pub fills: Vec<Fill>,

    pub unfilled_quote_qty: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct Fill {
    pub base_qty: u64,
    pub quote_qty: u64,
    pub user_qty: u64,
    pub fee_qty: u64,
    pub price: f64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MakerSimulation {
    pub total_quote_qty: u64,
    pub user_quote_qty: u64,
    pub fee_quote_qty: u64,
    pub limit_price: f64,

    pub full_quote_qty: u64,
    pub full_base_qty: u64,
    pub full_vwap: f64,
    pub full_vwar: f64,

    pub would_post: bool,
    pub depth: usize,
    pub posted_quote_qty: u64,
    pub posted_user_qty: u64,
    pub posted_fee_qty: u64,
    pub posted_base_qty: u64,
    pub posted_vwap: f64,
    pub posted_vwar: f64,
    pub preceding_base_qty: u64,
    pub preceding_quote_qty: u64,
    pub preceding_vwap: f64,
    pub preceding_vwar: f64,

    pub would_match: bool,
    pub self_match: bool,
    pub matches: usize,
    pub filled_quote_qty: u64,
    pub filled_user_qty: u64,
    pub filled_fee_qty: u64,
    pub filled_base_qty: u64,
    pub filled_vwap: f64,
    pub filled_vwar: f64,
    pub fills: Vec<Fill>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sample_liquidity() {
        let om = OrderbookModel {
            tenor: 24 * 60 * 60 * 180,
            origination_fee: 0,
            bids: vec![Order {
                owner: Default::default(),
                order_tag: Default::default(),
                base_size: 100,
                price: (9 << 32) / 10,
            }],
            asks: vec![],
        };

        let sample = om.sample_liquidity(Side::LoanOffer, None, None);

        assert_eq!(sample.side, Side::LoanOffer);
        assert_eq!(sample.total_quote_qty, 90);
        assert_eq!(sample.sample_quote_qty, sample.total_quote_qty);
        assert_eq!(sample.points[0].cumulative_rate, 0.2136);
    }

    #[test]
    fn test_sample_liquidity_2() {
        let om = OrderbookModel {
            tenor: 24 * 60 * 60,
            origination_fee: 0,
            bids: vec![Order {
                owner: Default::default(),
                order_tag: Default::default(),
                base_size: 10001469969,
                price: f64_to_fp32(0.9998630231351882),
            }],
            asks: vec![],
        };

        let sample = om.sample_liquidity(Side::LoanOffer, None, None);

        assert_eq!(sample.total_quote_qty, 10000100000);
        assert_eq!(sample.points[0].cumulative_rate, 0.05);
    }

    fn get_pubkey() -> Pubkey {
        Pubkey::new_from_array([
            1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4,
            3, 2, 1,
        ])
    }

    fn populate_orderbook_model() -> OrderbookModel {
        OrderbookModel {
            tenor: 60 * 60 * 24 * 90,
            origination_fee: 0,
            bids: vec![
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 1_000,
                    price: f64_to_fp32(0.96),
                },
                Order {
                    owner: get_pubkey(),
                    order_tag: OrderTag::default(),
                    base_size: 1_500,
                    price: f64_to_fp32(0.94),
                },
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 500,
                    price: f64_to_fp32(0.90),
                },
            ],
            asks: vec![
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 2_000,
                    price: f64_to_fp32(0.97),
                },
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 4_500,
                    price: f64_to_fp32(0.98),
                },
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 1_500,
                    price: f64_to_fp32(0.99),
                },
            ],
        }
    }

    #[test]
    fn test_misc() {
        let om = populate_orderbook_model();

        assert_eq!(om.loan_offers(), &om.bids);
        assert_eq!(om.orders_on("loanoffer".into()), om.loan_offers());

        assert_eq!(om.loan_requests(), &om.asks);
        assert_eq!(om.orders_on("loanrequest".into()), om.loan_requests());
    }

    #[test]
    fn test_would_match() {
        let om = populate_orderbook_model();

        assert!(om.would_match("lend".into(), f64_to_fp32(0.97)));
        assert!(om.would_match("lend".into(), f64_to_fp32(1.0)));
        assert!(!om.would_match("lend".into(), f64_to_fp32(0.9699)));

        assert!(om.would_match("borrow".into(), f64_to_fp32(0.96)));
        assert!(om.would_match("borrow".into(), f64_to_fp32(0.50)));
        assert!(!om.would_match("borrow".into(), f64_to_fp32(0.960001)));
    }

    #[test]
    fn test_simulate_fills() {
        let om = populate_orderbook_model();

        let sim = om.simulate_taker("lend".into(), 7_000, None, None);
        assert_eq!(sim.matches, 3);
        assert_eq!(sim.fills[0].base_qty, 2_000);
        assert_eq!(sim.unfilled_quote_qty, 1); // NOTE Rounding
        assert_eq!(sim.filled_vwap, 0.9777870913663035);
        assert!(!sim.self_match);
    }

    #[test]
    fn test_simulate_fills_self_match() {
        let om = populate_orderbook_model();

        let action = "borrow".into();
        let quote_qty = 2_000;
        let limit_price = None;
        let user = Some(get_pubkey());

        let sim = om.simulate_taker(action, quote_qty, limit_price, user);
        assert!(sim.self_match);
    }

    #[test]
    fn test_simulate_queuing() {
        let om = populate_orderbook_model();

        let sim = om.simulate_maker("lend".into(), 1_000, f64_to_fp32(0.94), None);
        assert_eq!(sim.depth, 2);
        assert_eq!(sim.preceding_base_qty, 2_500);
        assert_eq!(sim.preceding_quote_qty, 2_370);
        assert_eq!(sim.preceding_vwap, 0.948);
    }

    #[test]
    fn test_simulate_maker() {
        let om = populate_orderbook_model();

        let sim = om.simulate_maker("lend".into(), 8_500, f64_to_fp32(0.99), None);
        assert_eq!(sim.matches, 3);
        assert_eq!(sim.fills[0].base_qty, 2_000);
        assert_eq!(sim.filled_vwap, 0.979);
        assert!(!sim.self_match);
        assert!(sim.would_post);
        println!("{:?}", sim.fills);
        assert_eq!(sim.posted_quote_qty, 668);
    }

    #[test]
    fn test_refresh_from_snapshot() {
        let bids = vec![Order {
            owner: Pubkey::default(),
            order_tag: OrderTag::default(),
            base_size: 123,
            price: 456,
        }];
        let asks = vec![Order {
            owner: Pubkey::default(),
            order_tag: OrderTag::default(),
            base_size: 789,
            price: 101112,
        }];

        let mut om = OrderbookModel {
            tenor: 11,
            origination_fee: 0,
            bids: vec![],
            asks: vec![],
        };

        om.refresh_from_snapshot(OrderbookSnapshot {
            bids: bids.clone(),
            asks: asks.clone(),
        });

        assert_eq!(om.bids, bids);
        assert_eq!(om.asks, asks);
    }
}

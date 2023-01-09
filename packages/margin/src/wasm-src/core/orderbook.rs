use agnostic_orderbook::state::critbit::Slab;
use bonfida_utils::fp_math::{fp32_div, fp32_mul_ceil, fp32_mul_floor};
use jet_fixed_term::orderbook::state::{CallbackInfo, OrderTag};
use serde::Serialize;
use solana_program::pubkey::Pubkey;

use crate::orderbook::interest_pricing::{f64_to_fp32, fp32_to_f64};
use crate::orderbook::methods::{base_to_quote, price_to_rate};

pub struct OrderbookModel {
    tenor: u64,
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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

impl OrderbookModel {
    pub fn new(tenor: u64) -> Self {
        Self {
            tenor,
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
                        owner: callback.owner,
                        order_tag: callback.order_tag,
                        base_size: leaf.base_quantity,
                        price: leaf.price(),
                    }
                })
                .collect()
        };

        self.bids = extract_orders(bids_buffer, false);
        self.asks = extract_orders(asks_buffer, true);
    }

    // TODO Interpolate on a set of points instead
    pub fn sample_liquidity(&self, side: Side) -> LiquiditySample {
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
            // TODO adjust for Side
            let quote_size = base_to_quote(base_size, limit_price);
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

        if !points.is_empty() {
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

    pub fn would_match(&self, action: Action, limit_price: u64) -> bool {
        let orders = self.orders_on(Side::matching(action));

        if orders.is_empty() {
            false
        } else {
            orders[0].matches(action, limit_price)
        }
    }

    // TODO Alert self match
    // TODO Don't panic
    pub fn simulate_fills(
        &self,
        action: Action,
        quote_qty: u64,
        limit_price: Option<u64>,
    ) -> FillSimulation {
        let limit_price = limit_price.unwrap_or_else(|| action.worst_price());
        let side = Side::matching(action);

        let mut filled_base_qty = 0;
        let mut unfilled_quote_qty = quote_qty;
        let mut fills = vec![];
        for order in self.orders_on(side) {
            if unfilled_quote_qty > 0 && order.matches(action, limit_price) {
                let maker_base_qty = order.base_size;
                let unfilled_base_qty = fp32_div(unfilled_quote_qty, order.price).unwrap();
                let fill_base_qty = maker_base_qty.min(unfilled_base_qty);
                let fill_quote_qty = side.base_to_quote(fill_base_qty, order.price).unwrap();

                let fill = Fill {
                    base_qty: fill_base_qty,
                    quote_qty: fill_quote_qty,
                    price: fp32_to_f64(order.price),
                };

                fills.push(fill);

                filled_base_qty += fill_base_qty;
                unfilled_quote_qty -= fill_quote_qty;
            } else {
                break;
            }
        }

        let filled_quote_qty = quote_qty - unfilled_quote_qty;
        let vwap = if filled_base_qty > 0 {
            filled_quote_qty as f64 / filled_base_qty as f64
        } else {
            f64::NAN
        };
        let vwar = if vwap.is_normal() {
            price_to_rate(f64_to_fp32(vwap), self.tenor) as f64 / 10_000_f64
        } else {
            f64::NAN
        };

        FillSimulation {
            limit_price: fp32_to_f64(limit_price),
            order_quote_qty: quote_qty,
            filled_quote_qty,
            unfilled_quote_qty,
            filled_base_qty,
            unfilled_base_qty: fp32_div(unfilled_quote_qty, limit_price).unwrap(),
            matches: fills.len(),
            vwap,
            vwar,
            fills,
        }
    }

    pub fn simulate_queuing(&self, action: Action, limit_price: u64) -> QueuingSimulation {
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

        let vwap = if preceding_quote_qty > 0 {
            preceding_quote_qty as f64 / preceding_base_qty as f64
        } else {
            f64::NAN
        };
        let vwar = if vwap.is_normal() {
            price_to_rate(f64_to_fp32(vwap), self.tenor) as f64 / 10_000_f64
        } else {
            f64::NAN
        };

        QueuingSimulation {
            depth,
            preceding_base_qty,
            preceding_quote_qty,
            vwap,
            vwar,
        }
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
pub struct FillSimulation {
    pub limit_price: f64,
    pub order_quote_qty: u64,
    pub filled_quote_qty: u64,
    pub unfilled_quote_qty: u64,
    pub filled_base_qty: u64,
    pub unfilled_base_qty: u64,
    pub matches: usize,
    pub vwap: f64,
    pub vwar: f64,
    pub fills: Vec<Fill>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Fill {
    pub base_qty: u64,
    pub quote_qty: u64,
    pub price: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct QueuingSimulation {
    pub depth: usize,
    pub preceding_base_qty: u64,
    pub preceding_quote_qty: u64,
    pub vwap: f64,
    pub vwar: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sample_liquidity() {
        let om = OrderbookModel {
            tenor: 24 * 60 * 60 * 180,
            bids: vec![Order {
                owner: Default::default(),
                order_tag: Default::default(),
                base_size: 100,
                price: (9 << 32) / 10,
            }],
            asks: vec![],
        };

        let sample = om.sample_liquidity(Side::LoanOffer);

        assert_eq!(sample.side, Side::LoanOffer);
        assert_eq!(sample.total_quote_qty, 89); // rounding
        assert_eq!(sample.sample_quote_qty, sample.total_quote_qty);
        assert_eq!(sample.points[0].cumulative_rate, 0.2363);
    }

    #[test]
    fn test_sample_liquidity_2() {
        let om = OrderbookModel {
            tenor: 24 * 60 * 60,
            bids: vec![Order {
                owner: Default::default(),
                order_tag: Default::default(),
                base_size: 10001469970,
                price: f64_to_fp32(0.9998630231351882),
            }],
            asks: vec![],
        };

        let sample = om.sample_liquidity(Side::LoanOffer);

        assert_eq!(sample.total_quote_qty, 10000100000);
        assert_eq!(sample.points[0].cumulative_rate, 0.05);
    }

    fn populate_orderbook_model() -> OrderbookModel {
        OrderbookModel {
            tenor: 60 * 60 * 24 * 90,
            bids: vec![
                Order {
                    owner: Pubkey::default(),
                    order_tag: OrderTag::default(),
                    base_size: 1_000,
                    price: f64_to_fp32(0.96),
                },
                Order {
                    owner: Pubkey::default(),
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

        let sim = om.simulate_fills("lend".into(), 7_000, None);
        assert_eq!(sim.matches, 3);
        assert_eq!(sim.fills[0].base_qty, 2_000);
        assert_eq!(sim.unfilled_quote_qty, 1); // NOTE Rounding
        assert_eq!(sim.unfilled_base_qty, 1); // NOTE Rounding
        assert_eq!(sim.vwap, 0.9777870913663035);
    }

    #[test]
    fn test_simulate_queuing() {
        let om = populate_orderbook_model();

        let sim = om.simulate_queuing("lend".into(), f64_to_fp32(0.94));
        assert_eq!(sim.depth, 2);
        assert_eq!(sim.preceding_base_qty, 2_500);
        assert_eq!(sim.preceding_quote_qty, 2_370);
        assert_eq!(sim.vwap, 0.948);
    }
}

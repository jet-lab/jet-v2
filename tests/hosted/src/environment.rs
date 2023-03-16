//! Test helpers for the `environment` crate.  
//! Defines sane defaults that can be used by most tests.

use jet_environment::config::{FixedTermMarketConfig, TokenDescription};
use jet_margin_pool::{MarginPoolConfig, PoolFlags};

use crate::{test_default, TestDefault};

/// High level token definition to simplify test setup when you only care about:
/// - token name
/// - whether there is a lending pool
/// - tenors for any fixed term markets, if any
///
/// Easily converted into a TokenDescription for use with TestContext.
///
/// If you have more complex requirements for your test, you may want to
/// manually create the TokenDescriptions with assistance from the
/// `test_default()` function to fill in the fields you don't care about.
#[derive(Clone)]
pub struct TestToken {
    pub name: String,
    pub margin_pool: bool,
    pub fixed_term_tenors: Vec<u64>,
}

impl From<TestToken> for TokenDescription {
    fn from(value: TestToken) -> Self {
        TokenDescription {
            symbol: value.name.clone(),
            name: value.name,
            margin_pool: value.margin_pool.then_some(test_default()),
            fixed_term_markets: value
                .fixed_term_tenors
                .into_iter()
                .map(|tenor| FixedTermMarketConfig {
                    borrow_tenor: tenor,
                    lend_tenor: tenor,
                    ..test_default()
                })
                .collect(),
            ..test_default()
        }
    }
}

impl TestToken {
    /// just a token - no pool or fixed term
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            margin_pool: false,
            fixed_term_tenors: vec![],
        }
    }

    /// token with a pool - no fixed term
    pub fn with_pool(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            margin_pool: true,
            fixed_term_tenors: vec![],
        }
    }

    pub fn description(self) -> TokenDescription {
        self.into()
    }
}

impl TestDefault for FixedTermMarketConfig {
    fn test_default() -> Self {
        FixedTermMarketConfig {
            borrow_tenor: 3600,
            lend_tenor: 3600,
            origination_fee: 0,
            min_order_size: 100,
            paused: false,
            ticket_price: Some(0.9),
            ticket_collateral_weight: 90,
            ticket_pyth_price: None,
            ticket_pyth_product: None,
        }
    }
}

impl TestDefault for MarginPoolConfig {
    fn test_default() -> Self {
        MarginPoolConfig {
            borrow_rate_0: 10,
            borrow_rate_1: 20,
            borrow_rate_2: 30,
            borrow_rate_3: 40,
            utilization_rate_1: 10,
            utilization_rate_2: 20,
            management_fee_rate: 10,
            flags: PoolFlags::ALLOW_LENDING.bits(),
            reserved: 0,
        }
    }
}

impl TestDefault for TokenDescription {
    fn test_default() -> Self {
        TokenDescription {
            name: String::from("Default"),
            symbol: String::from("Default"),
            decimals: Some(6),
            precision: 6,
            mint: None,
            pyth_price: None,
            pyth_product: None,
            max_test_amount: None,
            collateral_weight: 100,
            max_leverage: 20_00,
            margin_pool: None,
            fixed_term_markets: vec![],
        }
    }
}

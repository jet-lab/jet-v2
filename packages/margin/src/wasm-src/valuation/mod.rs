//! Margin account valuation and forecasting

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub const MARGIN_ACCOUNT_SETUP_LEVERAGE_FRACTION: f64 = 0.5;

#[derive(Serialize, Deserialize)]
pub struct MarginAccountValuationInput {
    pub positions: HashMap<String, MarginPosition>,
    pub changes: Vec<MarginPosition>,
    pub prices: HashMap<String, OraclePrice>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MarginAccountValuation {
    pub liabilities: f64,
    // See https://github.com/rustwasm/wasm-bindgen/issues/1818
    #[wasm_bindgen(js_name = requiredCollateral)]
    pub required_collateral: f64,
    #[wasm_bindgen(js_name = requiredSetupCollateral)]
    pub required_setup_collateral: f64,
    #[wasm_bindgen(js_name = weightedCollateral)]
    pub weighted_collateral: f64,
    #[wasm_bindgen(js_name = effectiveCollateral)]
    pub effective_collateral: f64,
    #[wasm_bindgen(js_name = availableCollateral)]
    pub available_collateral: f64,
    #[wasm_bindgen(js_name = availableSetupCollateral)]
    pub available_setup_collateral: f64,
    pub assets: f64,
    #[wasm_bindgen(js_name = totalPositions)]
    pub total_positions: u32,
    #[wasm_bindgen(js_name = unvaluedPositions)]
    pub unvalued_positions: u32,
}

#[wasm_bindgen]
impl MarginAccountValuation {
    #[wasm_bindgen(constructor)]
    pub fn new(val: JsValue) -> Result<MarginAccountValuation, JsError> {
        let MarginAccountValuationInput {
            positions,
            changes,
            prices,
        } = serde_wasm_bindgen::from_value(val)?;

        Ok(Self::value(positions, changes, &prices))
    }

    #[wasm_bindgen]
    pub fn setup_leverage() -> f64 {
        MARGIN_ACCOUNT_SETUP_LEVERAGE_FRACTION
    }

    fn value(
        positions: HashMap<String, MarginPosition>,
        changes: Vec<MarginPosition>,
        prices: &HashMap<String, OraclePrice>,
    ) -> Self {
        // Update positions with changes
        let mut updated = positions;
        for change in changes {
            let position = updated.get_mut(&change.address);
            if let Some(position) = position {
                position.balance += change.balance;
            } else {
                updated.insert(change.address.clone(), change);
            }
        }
        let mut valuation = MarginAccountValuation {
            assets: 0.0,
            liabilities: 0.0,
            required_collateral: 0.0,
            weighted_collateral: 0.0,
            effective_collateral: 0.0,
            available_collateral: 0.0,
            required_setup_collateral: 0.0,
            available_setup_collateral: 0.0,
            total_positions: updated.len() as u32,
            unvalued_positions: 0,
        };
        for position in updated.values() {
            let price = prices.get(&position.token);
            if let Some(price) = price {
                let value =
                    position.balance as f64 * 10.0_f64.powi(position.exponent) * price.price;
                match &position.position_kind {
                    // 0 = NoValue
                    // 1 = Deposit
                    // 3 = AdapterCollateral
                    1 => {
                        valuation.assets += value;
                        valuation.weighted_collateral += value * position.value_modifier;
                    }
                    3 => {
                        valuation.assets += value;
                        valuation.weighted_collateral += value * position.value_modifier;
                    }
                    // 2 = Claim
                    2 => {
                        valuation.liabilities += value;
                        valuation.required_collateral += value / position.value_modifier;
                        valuation.required_setup_collateral += value
                            / (position.value_modifier * MARGIN_ACCOUNT_SETUP_LEVERAGE_FRACTION);
                    }
                    _ => {}
                }
            } else {
                valuation.unvalued_positions += 1;
            }
        }

        valuation.effective_collateral = valuation.weighted_collateral - valuation.liabilities;
        valuation.available_collateral =
            valuation.weighted_collateral - valuation.liabilities - valuation.required_collateral;
        valuation.available_setup_collateral = valuation.weighted_collateral
            - valuation.liabilities
            - valuation.required_setup_collateral;

        valuation
    }
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct OraclePrice {
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[wasm_bindgen(getter_with_clone)]
pub struct MarginPosition {
    pub address: String,
    pub token: String,
    // So we can have negative balances when subtracting
    pub balance: i64,
    pub exponent: i32,
    pub position_kind: u8,
    pub value_modifier: f64,
}

#[wasm_bindgen]
impl MarginPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(
        address: String,
        token: String,
        balance: i64,
        exponent: i32,
        position_kind: u8,
        value_modifier: f64,
    ) -> Self {
        Self {
            address,
            token,
            balance,
            exponent,
            position_kind,
            value_modifier,
        }
    }
}

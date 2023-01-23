use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use hosted_tests::load::{
    under_collateralized_fixed_term_borrow_orders, UnhealthyAccountsLoadTestScenario,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    under_collateralized_fixed_term_borrow_orders(UnhealthyAccountsLoadTestScenario {
        liquidator: Pubkey::from_str("6xfNgz63mk5y8K5CSiNfiBN5DCNMj9s9BwNwJznnfHB7").unwrap(),
        ..Default::default()
    })
    .await
    .unwrap()
}

use hosted_tests::load::{
    under_collateralized_fixed_term_borrow_orders, UnhealthyAccountsLoadTestScenario,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    under_collateralized_fixed_term_borrow_orders(UnhealthyAccountsLoadTestScenario {
        ..Default::default()
    })
    .await
    .unwrap()
}

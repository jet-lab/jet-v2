use hosted_tests::load::{unhealthy_accounts_load_test, UnhealthyAccountsLoadTestScenario};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    unhealthy_accounts_load_test(UnhealthyAccountsLoadTestScenario {
        // user_count: todo!(),
        // mint_count: todo!(),
        // repricing_delay: todo!(),
        // repricing_scale: todo!(),
        ..Default::default()
    })
    .await
    .unwrap()
}

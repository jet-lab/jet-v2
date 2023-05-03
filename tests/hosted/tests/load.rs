use hosted_tests::load::{
    under_collateralized_fixed_term_borrow_orders, unhealthy_accounts_load_test,
    UnhealthyAccountsLoadTestScenario,
};
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn pools_load_test_can_run() -> Result<(), anyhow::Error> {
    unhealthy_accounts_load_test(UnhealthyAccountsLoadTestScenario {
        keep_looping: false,
        user_count: 1,
        mint_count: 1,
        repricing_delay: 0,
        repricing_scale: 0.9,
        liquidator: Keypair::new().pubkey(),
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn fixed_term_load_test_can_run() -> Result<(), anyhow::Error> {
    under_collateralized_fixed_term_borrow_orders(UnhealthyAccountsLoadTestScenario {
        keep_looping: false,
        user_count: 1,
        mint_count: 1,
        repricing_delay: 0,
        repricing_scale: 0.9,
        liquidator: Keypair::new().pubkey(),
    })
    .await
}

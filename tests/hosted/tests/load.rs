use hosted_tests::load::{unhealthy_accounts_load_test, UnhealthyAccountsLoadTestScenario};
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test(flavor = "multi_thread")]
async fn trivial_load_test_execution() -> Result<(), anyhow::Error> {
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

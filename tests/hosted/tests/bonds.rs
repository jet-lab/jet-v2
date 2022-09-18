use hosted_tests::{
    bonds::{BondsTestManager, MarketConfig},
    context::test_context,
};
use jet_simulation::generate_keypair;
use solana_sdk::signer::Signer;

const TOKEN_DECIMALS: u8 = 6;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn full_direct() -> Result<(), anyhow::Error> {
    // Get the mocked runtime
    let ctx = test_context().await;
    let usdc = ctx.tokens.create_token(TOKEN_DECIMALS, None, None).await?;
    let authority = generate_keypair();
    let config = MarketConfig {
        authority: authority.pubkey(),
        token_mint: usdc,
        seed: [0u8; 32],
        event_queue: generate_keypair(),
        bids: generate_keypair(),
        asks: generate_keypair(),
        event_queue_capacity: 1_000,
        orderbook_capacity: 1_000,
    };
    let mut manager = BondsTestManager::new(ctx.rpc.clone()).await;
    let usdc_market = manager.markets.create_test_market(config);
    Ok(())
}

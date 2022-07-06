use std::num::NonZeroU64;

use anyhow::Error;

use jet_margin_sdk::ix_builder::{OrderParams, OrderSide, OrderType, SelfTradeBehavior};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
    serum::SerumClient,
    tokens::TokenPrice,
};

use jet_margin_pool::{MarginPoolConfig, PoolFlags};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;

const ONE_USDC: u64 = 1_000_000;
const ONE_TSOL: u64 = LAMPORTS_PER_SOL;

const DEFAULT_POOL_CONFIG: MarginPoolConfig = MarginPoolConfig {
    borrow_rate_0: 10,
    borrow_rate_1: 20,
    borrow_rate_2: 30,
    borrow_rate_3: 40,
    utilization_rate_1: 10,
    utilization_rate_2: 20,
    management_fee_rate: 10,
    management_fee_collect_threshold: 100,
    flags: PoolFlags::ALLOW_LENDING.bits(),
};

struct TestEnv {
    usdc: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_fees = ctx
        .tokens
        .create_account(&usdc, &ctx.authority.pubkey())
        .await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_fees = ctx
        .tokens
        .create_account(&tsol, &ctx.authority.pubkey())
        .await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

    let pools = [
        MarginPoolSetupInfo {
            token: usdc,
            fee_destination: usdc_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 1_00,
            max_leverage: 10_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdc_oracle,
        },
        MarginPoolSetupInfo {
            token: tsol,
            fee_destination: tsol_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 95,
            max_leverage: 4_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: tsol_oracle,
        },
    ];

    for pool_info in pools {
        ctx.margin.create_pool(&pool_info).await?;
    }

    Ok(TestEnv { usdc, tsol })
}

#[tokio::test(flavor = "multi_thread")]
async fn serum_swap() -> Result<(), anyhow::Error> {
    // Get the mocked runtime
    let ctx = test_context().await;
    let env = setup_environment(ctx).await?;

    // Create our two user wallets, with some SOL funding to get started
    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user_a = ctx.margin.user(&wallet_a).await?;
    let user_b = ctx.margin.user(&wallet_b).await?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;

    // Create a serum market
    let serum_client =
        SerumClient::create_market(ctx.rpc.clone(), env.tsol, env.usdc, 100, 1000).await?;

    let usdc_transit_source_a = ctx
        .tokens
        .create_account(&env.usdc, user_a.address())
        .await?;
    let tsol_transit_target_a = ctx
        .tokens
        .create_account(&env.tsol, user_a.address())
        .await?;
    let usdc_transit_target_a = ctx
        .tokens
        .create_account(&env.usdc, user_a.address())
        .await?;
    let tsol_transit_source_a = ctx
        .tokens
        .create_account(&env.tsol, user_a.address())
        .await?;

    let tsol_transit_source_b = ctx
        .tokens
        // multiplied by the coin lot size
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), ONE_TSOL / 10 * 9 * 100)
        .await?;

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000 * ONE_USDC)
        .await?;
    let user_a_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_a.pubkey(), 100 * ONE_TSOL)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10 * ONE_TSOL)
        .await?;

    // Set the prices for each token
    ctx.tokens
        .set_price(
            // Set price to 1 USD +- 0.01
            &env.usdc,
            &TokenPrice {
                exponent: -8,
                price: 100_000_000,
                confidence: 1_000_000,
                twap: 100_000_000,
            },
        )
        .await?;
    ctx.tokens
        .set_price(
            // Set price to 100 USD +- 1
            &env.tsol,
            &TokenPrice {
                exponent: -8,
                price: 10_000_000_000,
                confidence: 100_000_000,
                twap: 10_000_000_000,
            },
        )
        .await?;

    // Deposit user funds into their margin accounts
    user_a
        .deposit(&env.usdc, &user_a_usdc_account, 1_000 * ONE_USDC)
        .await?;
    user_a
        .deposit(&env.tsol, &user_a_tsol_account, 10 * ONE_TSOL)
        .await?;
    user_b
        .deposit(&env.tsol, &user_b_tsol_account, 10 * ONE_TSOL)
        .await?;

    // // Verify user tokens have been deposited
    assert_eq!(0, ctx.tokens.get_balance(&user_a_usdc_account).await?);
    assert_eq!(
        90 * ONE_TSOL,
        ctx.tokens.get_balance(&user_a_tsol_account).await?
    );
    assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    let user_a_open_orders = user_a.init_open_orders(serum_client.market(), None).await?;
    let user_b_open_orders = user_b
        .init_open_orders(serum_client.market(), Some(wallet_b.pubkey()))
        .await?;

    // User B places a limit order, so that user B can buy at market
    user_b
        .new_spot_order(
            serum_client.market(),
            user_b_open_orders,
            tsol_transit_source_b,
            OrderParams {
                side: OrderSide::Ask,
                limit_price: NonZeroU64::new(1).unwrap(),
                max_base_qty: NonZeroU64::new(ONE_TSOL / 10 * 9).unwrap(),
                max_native_quote_qty_including_fees: NonZeroU64::new(u64::MAX).unwrap(),
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                order_type: OrderType::Limit,
                client_order_id: 1,
                limit: u16::MAX,
            },
        )
        .await?;

    // Now user A swaps their USDC for TSOL
    user_a
        .serum_swap(
            // &env.usdc,
            // &env.tsol,
            serum_client.market(),
            user_a_open_orders,
            usdc_transit_source_a,
            tsol_transit_target_a,
            100 * ONE_USDC,
            // we want a minimum of 0.9 SOL for 100 USDC
            ONE_TSOL / 10 * 9,
            true,
        )
        .await?;

    // // Verify that swap has taken place in the pool
    // assert_eq!(
    //     // There was 1 million USDC as a start
    //     1_000_100 * ONE_USDC,
    //     ctx.tokens.get_balance(&swap_pool.token_a).await?
    // );
    // assert!(
    //     // Pool balance less almost 1 SOL
    //     10_000 * ONE_TSOL - 900_000_000 >= ctx.tokens.get_balance(&swap_pool.token_b).await?
    // );

    // Swap in a different order
    // Now user A swaps their USDC for TSOL
    user_a
        .serum_swap(
            // &env.usdc,
            // &env.tsol,
            serum_client.market(),
            user_a_open_orders,
            tsol_transit_target_a,
            usdc_transit_source_a,
            2 * ONE_TSOL,
            180 * ONE_USDC,
            true,
        )
        .await?;

    // // Verify that swap has taken place in the pool
    // assert!(
    //     1_000_100 * ONE_USDC - 180 * ONE_USDC >= ctx.tokens.get_balance(&swap_pool.token_a).await?
    // );
    // assert!(
    //     (10_000 * ONE_TSOL - 900_000_000) + 2 * ONE_TSOL
    //         >= ctx.tokens.get_balance(&swap_pool.token_b).await?
    // );

    Ok(())
}

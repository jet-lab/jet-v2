use std::num::NonZeroU64;

use anyhow::Error;

use jet_margin_sdk::{
    ix_builder::{OrderParams, OrderSide, OrderType, SelfTradeBehavior, SwapParams},
    tokens::TokenPrice,
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
    serum::SerumClient,
};

use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
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
    reserved: 0,
    flags: PoolFlags::ALLOW_LENDING.bits(),
};

struct TestEnv {
    usdc: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

    let pools = [
        MarginPoolSetupInfo {
            token: usdc,
            token_kind: TokenKind::Collateral,
            collateral_weight: 1_00,
            max_leverage: 10_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdc_oracle,
        },
        MarginPoolSetupInfo {
            token: tsol,
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
    let wallet_c = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user_a = ctx.margin.user(&wallet_a, 0).await?;
    let user_b = ctx.margin.user(&wallet_b, 0).await?;
    let user_c = ctx.margin.user(&wallet_c, 0).await?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;

    let base_lot_size = 1_000;
    let quote_lot_size = 10;

    // Create a serum market
    let serum_client = SerumClient::create_market(
        ctx.rpc.clone(),
        env.tsol,
        env.usdc,
        base_lot_size,
        quote_lot_size,
    )
    .await?;

    let usdc_transit_a = ctx
        .tokens
        .create_account(&env.usdc, user_a.address())
        .await?;
    let tsol_transit_a = ctx
        .tokens
        .create_account(&env.tsol, user_a.address())
        .await?;

    // User B funds their TSOL transit account with 10 SOL
    let tsol_transit_b = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10 * ONE_TSOL)
        .await?;
    // User C funds their USDC transit account with 100 USDC
    let usdc_transit_c = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_c.pubkey(), 1000 * ONE_USDC)
        .await?;

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 10_000 * ONE_USDC)
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
        .deposit(
            &env.usdc,
            &user_a_usdc_account,
            TokenChange::shift(10_000 * ONE_USDC),
        )
        .await?;
    user_a
        .deposit(
            &env.tsol,
            &user_a_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.tsol,
            &user_b_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
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
    let user_c_open_orders = user_c
        .init_open_orders(serum_client.market(), Some(wallet_c.pubkey()))
        .await?;

    // User B places a limit order, so that user A can swap at market
    user_b
        .new_spot_order(
            serum_client.market(),
            user_b_open_orders,
            tsol_transit_b,
            OrderParams {
                side: OrderSide::Ask,
                limit_price: NonZeroU64::new(price_number_to_lot(
                    10,
                    ONE_TSOL,
                    ONE_USDC,
                    base_lot_size,
                    quote_lot_size,
                ))
                .unwrap(), // what if the price is in quote with decimals?
                // 10 SOL divided by coin lot size
                max_base_qty: NonZeroU64::new(10 * ONE_TSOL / base_lot_size).unwrap(),
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
            serum_client.market(),
            user_a_open_orders,
            tsol_transit_a,
            usdc_transit_a,
            SwapParams {
                // we want a minimum of 0.95 SOL for 100 USDC
                amount_in: 100 * ONE_USDC,
                minimum_amount_out: 95 * ONE_TSOL / 10,
                swap_direction: jet_margin_swap::instructions::SwapDirection::Bid,
            },
        )
        .await?;

    // User B places a limit order, so that user A can swap at market
    user_c
        .new_spot_order(
            serum_client.market(),
            user_c_open_orders,
            usdc_transit_c,
            OrderParams {
                side: OrderSide::Bid,
                limit_price: NonZeroU64::new(price_number_to_lot(
                    10,
                    ONE_TSOL,
                    ONE_USDC,
                    base_lot_size,
                    quote_lot_size,
                ))
                .unwrap(),
                max_base_qty: NonZeroU64::new(u64::MAX).unwrap(),
                max_native_quote_qty_including_fees: NonZeroU64::new(
                    1000 * ONE_USDC / quote_lot_size,
                )
                .unwrap(),
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                order_type: OrderType::Limit,
                client_order_id: 2,
                limit: u16::MAX,
            },
        )
        .await?;

    // Swap in a different order
    // Now user A swaps their USDC for TSOL
    user_a
        .serum_swap(
            serum_client.market(),
            user_a_open_orders,
            tsol_transit_a,
            usdc_transit_a,
            SwapParams {
                amount_in: 95 * ONE_TSOL / 10,
                minimum_amount_out: 100 * ONE_USDC,
                swap_direction: jet_margin_swap::instructions::SwapDirection::Ask,
            },
        )
        .await?;

    // TODO: check token balances after swaps

    Ok(())
}

fn price_number_to_lot(
    price: u64,
    base_lamports: u64,
    quote_lamports: u64,
    base_lot_size: u64,
    quote_lot_size: u64,
) -> u64 {
    price * (quote_lamports * base_lot_size) / (base_lamports * quote_lot_size)
}

// #[test]
// fn test_price_number_to_lot() {
//     let base_lot_size = 1_000;
//     let quote_lot_size = 100;
//     let base_decimal = 9;
//     let quote_decimal = 6;
//     let price = 40_000_000;

//     let lot_price = price_number_to_lot(price, base_decimal, quote_decimal, base_lot_size, quote_lot_size);
//     assert_eq!(lot_price, 400_000);
// }

// use anyhow::Error;

// use jet_margin_sdk::tokens::TokenPrice;
// use solana_sdk::native_token::LAMPORTS_PER_SOL;
// use solana_sdk::pubkey::Pubkey;
// use solana_sdk::signature::Signer;

// use hosted_tests::{
//     context::{test_context, MarginTestContext},
//     margin::MarginPoolSetupInfo,
// };

// use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
// use jet_metadata::TokenKind;
// use jet_simulation::create_wallet;

// const ONE_USDC: u64 = 1_000_000;
// const ONE_TSOL: u64 = LAMPORTS_PER_SOL;

// const DEFAULT_POOL_CONFIG: MarginPoolConfig = MarginPoolConfig {
//     borrow_rate_0: 10,
//     borrow_rate_1: 20,
//     borrow_rate_2: 30,
//     borrow_rate_3: 40,
//     utilization_rate_1: 10,
//     utilization_rate_2: 20,
//     management_fee_rate: 10,
//     flags: PoolFlags::ALLOW_LENDING.bits(),
//     reserved: 0,
// };

// struct TestEnv {
//     usdc: Pubkey,
//     tsol: Pubkey,
// }

// async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
//     let usdc = ctx.tokens.create_token(6, None, None).await?;
//     let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
//     let tsol = ctx.tokens.create_token(9, None, None).await?;
//     let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

//     let pools = [
//         MarginPoolSetupInfo {
//             token: usdc,
//             token_kind: TokenKind::Collateral,
//             collateral_weight: 1_00,
//             max_leverage: 4_00,
//             config: DEFAULT_POOL_CONFIG,
//             oracle: usdc_oracle,
//         },
//         MarginPoolSetupInfo {
//             token: tsol,
//             token_kind: TokenKind::Collateral,
//             collateral_weight: 95,
//             max_leverage: 4_00,
//             config: DEFAULT_POOL_CONFIG,
//             oracle: tsol_oracle,
//         },
//     ];

//     for pool_info in pools {
//         ctx.margin.create_pool(&pool_info).await?;
//     }

//     Ok(TestEnv { usdc, tsol })
// }

// /// Pool repayment test
// ///
// /// Tests that users can repay a claim by exhausting their deposit
// /// The test creates 2 users:
// /// 1. Deposits Token A
// /// 2. Deposits Token B, borrows Token A and tries to repay more than they have in deposit
// #[tokio::test(flavor = "multi_thread")]
// async fn pool_repay_allowable_max() -> Result<(), anyhow::Error> {
//     // Get the mocked runtime
//     let ctx = test_context().await;

//     let env = setup_environment(ctx).await?;

//     // Create our two user wallets, with some SOL funding to get started
//     let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
//     let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

//     // Create the user context helpers, which give a simple interface for executing
//     // common actions on a margin account
//     let user_a = ctx.margin.user(&wallet_a, 0)?;
//     let user_b = ctx.margin.user(&wallet_b, 0)?;

//     // Initialize the margin accounts for each user
//     user_a.create_account().await?;
//     user_b.create_account().await?;

//     // Create some tokens for each user to deposit
//     let user_a_usdc_account = ctx
//         .tokens
//         .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000_000 * ONE_USDC)
//         .await?;
//     let user_b_tsol_account = ctx
//         .tokens
//         .create_account_funded(&env.tsol, &wallet_b.pubkey(), 1_000 * ONE_TSOL)
//         .await?;
//     let user_b_usdc_account = ctx
//         .tokens
//         .create_account(&env.usdc, &wallet_b.pubkey())
//         .await?;

//     // Set the prices for each token
//     ctx.tokens
//         .set_price(
//             // Set price to 1 USD +- 0.01
//             &env.usdc,
//             &TokenPrice {
//                 exponent: -8,
//                 price: 100_000_000,
//                 confidence: 1_000_000,
//                 twap: 100_000_000,
//             },
//         )
//         .await?;
//     ctx.tokens
//         .set_price(
//             // Set price to 100 USD +- 1
//             &env.tsol,
//             &TokenPrice {
//                 exponent: -8,
//                 price: 10_000_000_000,
//                 confidence: 100_000_000,
//                 twap: 10_000_000_000,
//             },
//         )
//         .await?;

//     // Deposit user funds into their margin accounts
//     user_a
//         .deposit(
//             &env.usdc,
//             &user_a_usdc_account,
//             TokenChange::shift(1_000_000 * ONE_USDC),
//         )
//         .await?;
//     user_b
//         .deposit(
//             &env.tsol,
//             &user_b_tsol_account,
//             TokenChange::shift(1_000 * ONE_TSOL),
//         )
//         .await?;

//     // Verify user tokens have been deposited
//     assert_eq!(0, ctx.tokens.get_balance(&user_a_usdc_account).await?);
//     assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);

//     user_a.refresh_all_pool_positions().await?;
//     user_b.refresh_all_pool_positions().await?;

//     // User B borrows some USDC
//     user_b
//         .borrow(&env.usdc, TokenChange::shift(10_000 * ONE_USDC))
//         .await?;
//     // User B borrows an irrelevant amount
//     user_b
//         .withdraw(
//             &env.usdc,
//             &user_b_usdc_account,
//             TokenChange::shift(2_000 * ONE_USDC),
//         )
//         .await?;

//     user_a.refresh_all_pool_positions().await?;
//     user_b.refresh_all_pool_positions().await?;

//     // User repays their loan by setting the value to 0
//     user_b
//         .margin_repay(&env.usdc, TokenChange::shift(8_001 * ONE_USDC))
//         .await?;

//     // assert!(ctx.tokens.get_balance(&user_c_tsol_account).await? - 500 * ONE_TSOL < ONE_TSOL);

//     // // User C should be able to close all TSOL positions as loan is paid and deposit withdrawn
//     // user_c.close_token_positions(&env.tsol).await?;

//     Ok(())
// }

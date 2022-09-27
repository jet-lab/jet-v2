use anyhow::{Error, Result};

use jet_margin_sdk::tokens::TokenPrice;
use solana_sdk::clock::Clock;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
};

use jet_margin::TokenKind;
use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_simulation::{assert_custom_program_error, create_wallet};

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
    flags: PoolFlags::ALLOW_LENDING.bits(),
    reserved: 0,
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
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn rounding_poc() -> Result<()> {
    let ctx = test_context().await;
    let env = setup_environment(ctx).await?;

    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_c = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    let user_a = ctx.margin.user(&wallet_a, 0)?;
    let user_b = ctx.margin.user(&wallet_b, 0)?;
    let user_c = ctx.margin.user(&wallet_c, 0)?;

    user_a.create_account().await?;
    user_b.create_account().await?;
    user_c.create_account().await?;

    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 10_000_000 * ONE_USDC)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10_000 * ONE_TSOL)
        .await?;
    let user_c_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_c.pubkey(), 0)
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

    user_a
        .deposit(
            &env.usdc,
            &user_a_usdc_account,
            TokenChange::shift(5_000_000 * ONE_USDC),
        )
        .await?;
    user_b
        .deposit(
            &env.tsol,
            &user_b_tsol_account,
            TokenChange::shift(10_000 * ONE_TSOL),
        )
        .await?;

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    user_b
        .borrow(&env.usdc, TokenChange::shift(50000000000))
        .await?;

    let mut clk: Clock = match ctx.rpc.get_clock() {
        Some(c) => c,
        None => panic!("bad"),
    };

    // 1 second later...
    clk.unix_timestamp = 1;
    ctx.rpc.set_clock(clk);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // If the rounding is performed correctly, the user should try to burn 1 note,
    // and this should fail as they have no notes to burn.
    let withdraw_result = user_c
        .withdraw(&env.usdc, &user_c_usdc_account, TokenChange::shift(1))
        .await;

    // Should not succeed, there should be insufficient funds to burn notes
    assert_custom_program_error(
        anchor_spl::token::spl_token::error::TokenError::InsufficientFunds as u32,
        withdraw_result,
    );

    Ok(())
}

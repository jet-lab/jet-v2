use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use futures::future::{join_all, try_join_all};
use jet_margin_sdk::{
    fixed_term::{Crank, OrderParams},
    solana::transaction::InverseSendTransactionBuilder,
    util::asynchronous::MapAsync,
};
use jet_solana_client::transactions;
use std::{sync::Arc, time::Duration};

use crate::{
    fixed_term::{self, create_and_fund_fixed_term_market_margin_user, OrderAmount},
    margin_test_context,
    pricing::TokenPricer,
    setup_helper::{create_tokens, create_users, tokens},
    test_user::ONE,
};

pub struct UnhealthyAccountsLoadTestScenario {
    pub user_count: usize,
    pub mint_count: usize,
    pub repricing_delay: usize,
    pub repricing_scale: f64,
    pub keep_looping: bool,
    pub liquidator: Pubkey,
}

impl Default for UnhealthyAccountsLoadTestScenario {
    fn default() -> Self {
        Self {
            user_count: 2,
            mint_count: 2,
            repricing_delay: 1,
            repricing_scale: 0.999,
            keep_looping: true,
            liquidator: Pubkey::default(),
        }
    }
}

pub async fn unhealthy_accounts_load_test(
    scenario: UnhealthyAccountsLoadTestScenario,
) -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    let UnhealthyAccountsLoadTestScenario {
        user_count,
        mint_count,
        repricing_delay,
        repricing_scale,
        keep_looping,
        liquidator,
    } = scenario;
    ctx.margin_client()
        .set_liquidator_metadata(liquidator, true)
        .await?;
    println!("creating tokens");
    let (mut mints, _, pricer) = create_tokens(&ctx, mint_count).await?;
    println!("creating users");
    let mut users = create_users(&ctx, user_count + 1).await?;
    let big_depositor = users.pop().unwrap();
    println!("creating deposits");
    mints
        .iter()
        .map_async(|mint| big_depositor.deposit(mint, 1000 * ONE))
        .await?;
    users
        .iter()
        .zip(mints.iter().cycle())
        .map_async_chunked(16, |(user, mint)| user.deposit(mint, 100 * ONE))
        .await?;
    println!("creating loans");
    mints.rotate_right(mint_count / 2);
    users
        .iter()
        .zip(mints.iter().cycle())
        .map_async_chunked(32, |(user, mint)| user.borrow_to_wallet(mint, 80 * ONE))
        .await?;

    println!("incrementally lowering prices of half of the assets");
    let assets_to_devalue = mints[0..mints.len() / 2].to_vec();
    devalue_assets(
        1.0,
        pricer,
        assets_to_devalue,
        vec![], //todo!()
        keep_looping,
        repricing_scale,
        repricing_delay,
    )
    .await
}

pub async fn under_collateralized_fixed_term_borrow_orders(
    scenario: UnhealthyAccountsLoadTestScenario,
) -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    println!("creating fixed term market");
    let manager = Arc::new(fixed_term::TestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    println!("creating collateral token");
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let UnhealthyAccountsLoadTestScenario {
        user_count,
        mint_count: _, //todo
        repricing_delay,
        repricing_scale,
        keep_looping,
        liquidator,
    } = scenario;
    ctx.margin_client()
        .set_liquidator_metadata(liquidator, true)
        .await?;

    println!("creating users with collateral");
    let users = join_all((0..user_count).into_iter().map(|_| {
        create_and_fund_fixed_term_market_margin_user(
            &ctx,
            manager.clone(),
            vec![(collateral, 0, 350_000)],
        )
    }))
    .await;

    println!("creating borrow orders");
    try_join_all(users.iter().map(|user| async {
        transactions! {
            pricer.set_oracle_price_tx(&collateral, 1.0).await?,
            pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
            user.refresh_and_margin_borrow_order(underlying(1_000, 2_000)).await?,
        }
        .send_and_confirm_condensed_in_order(&client).await
    }))
    .await?;

    let crank = Crank::new(ctx.rpc(), &[manager.ix_builder.market()]).await?;
    tokio::spawn(crank.run_forever());

    println!("incrementally lowering prices of the collateral");
    devalue_assets(
        0.9,
        pricer,
        vec![collateral],
        vec![
            manager.ix_builder.token_mint(),
            manager.ix_builder.ticket_mint(),
        ],
        keep_looping,
        repricing_scale,
        repricing_delay,
    )
    .await
}

async fn devalue_assets(
    starting_price: f64,
    pricer: TokenPricer,
    assets_to_devalue: Vec<Pubkey>,
    assets_to_refresh: Vec<Pubkey>,
    keep_looping: bool,
    repricing_scale: f64,
    repricing_delay: usize,
) -> anyhow::Result<()> {
    println!("for assets {assets_to_devalue:?}...");
    let mut price = starting_price;
    loop {
        price *= repricing_scale;
        println!("setting price to {price}");
        for _ in 0..repricing_delay {
            pricer
                .set_prices(
                    assets_to_devalue
                        .iter()
                        .map(|&a| (a, price))
                        .chain(assets_to_refresh.iter().map(|&a| (a, 1.0)))
                        .collect(),
                    true,
                )
                .await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        if !keep_looping {
            return Ok(());
        }
    }
}

// todo dedupe with unit tests
fn underlying(quote: u64, rate_bps: u64) -> OrderParams {
    let borrow_amount = OrderAmount::from_quote_amount_rate(quote, rate_bps);
    OrderParams {
        max_ticket_qty: borrow_amount.base,
        max_underlying_token_qty: borrow_amount.quote,
        limit_price: borrow_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    }
}

use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use jet_margin_sdk::util::asynchronous::MapAsync;
use std::time::Duration;

use crate::{
    context::test_context,
    setup_helper::{create_tokens, create_users},
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
            repricing_delay: 0,
            repricing_scale: 0.999,
            keep_looping: true,
            liquidator: Pubkey::default(),
        }
    }
}

pub async fn unhealthy_accounts_load_test(
    scenario: UnhealthyAccountsLoadTestScenario,
) -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    let UnhealthyAccountsLoadTestScenario {
        user_count,
        mint_count,
        repricing_delay,
        repricing_scale,
        keep_looping: iterate,
        liquidator,
    } = scenario;
    ctx.margin.set_liquidator_metadata(liquidator, true).await?;
    println!("creating tokens");
    let (mut mints, _, pricer) = create_tokens(ctx, mint_count).await?;
    println!("creating users");
    let mut users = create_users(ctx, user_count + 1).await?;
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
    println!("for assets {assets_to_devalue:?}...");
    let mut price = 1.0;
    loop {
        price *= repricing_scale;
        let new_prices = assets_to_devalue
            .iter()
            .map(|mint| (*mint, price))
            .collect();
        println!("setting price to {price}");
        pricer.set_prices(new_prices, true).await?;
        for _ in 0..repricing_delay {
            std::thread::sleep(Duration::from_secs(1));
            // pricer.refresh_all_oracles().await?;
            pricer.set_prices(Vec::new(), true).await?;
        }
        if !iterate {
            return Ok(());
        }
    }
}

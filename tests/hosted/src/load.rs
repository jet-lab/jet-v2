use std::collections::{HashMap, HashSet};

use anchor_lang::{prelude::Pubkey, solana_program::example_mocks::solana_sdk::signature::Keypair};
use itertools::Itertools;
use jet_margin_pool::{ChangeKind, MarginPoolConfig, PoolFlags, TokenChange};
use jet_margin_sdk::{
    swap::SwapPool,
    tokens::{TokenOracle, TokenPrice},
};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;
use jet_static_program_registry::orca_swap_v2;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};

use crate::{
    context::{test_context, MarginTestContext},
    margin::{MarginPoolSetupInfo, MarginUser},
    setup_helper::{create_tokens, create_users, users},
    swap::SwapPoolConfig,
};

pub async fn load_test(user_count: usize, mint_count: usize) -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    let users = create_users(&ctx, user_count).await?;
    let (mut mints, _, pricer) = create_tokens(&ctx, mint_count).await?;
    // create deposits
    for (user, mint) in users.iter().zip(mints.iter().cycle()) {
        user.deposit(&mint, 100).await?;
    }
    // create loans
    mints.rotate_right(mint_count / 2);
    for (user, mint) in users.iter().zip(mints.iter().cycle()) {
        user.borrow_to_wallet(&mint, 10).await?;
    }
    // incrementally lower prices of half of the assets
    let assets_to_devalue = mints[0..mints.len() / 2].to_vec();
    let mut price = 1.0;
    for _ in 0..100 {
        price *= 0.99;
        for mint in assets_to_devalue.iter() {
            pricer.set_price(mint, price).await?;
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test() {
    load_test(40, 4).await.unwrap()
}

use std::collections::{HashSet, HashMap};

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
    swap::SwapPoolConfig,
};

struct Scenario {
    users: usize,
    mints: usize,
}

#[derive(Clone, Copy)]
struct Asset {
    mint: Pubkey,
    oracle: TokenOracle,
    vault: Pubkey,
}

enum SwapIdx {
    /// The first provided mint is in slot A
    A,
    /// The first provided mint is in slot B
    B
}

async fn load_test(scenario: Scenario) -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    let proctor = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    // create users
    let mut users = Vec::new();
    for _ in 0..scenario.users {
        let wallet = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
        let user = ctx.margin.user(&wallet, 0).await?;
        user.create_account().await?;
        users.push(user);
    }
    // create tokens
    let mut tokens: Vec<Asset> = Vec::new();
    for _ in 0..scenario.mints {
        let mint = ctx.tokens.create_token(6, None, None).await?;
        let oracle = ctx.tokens.create_oracle(&mint).await?;
        let pool_info = MarginPoolSetupInfo {
            token: mint,
            token_kind: TokenKind::Collateral,
            collateral_weight: 1_00_00,
            max_leverage: 10_00,
            config: MarginPoolConfig {
                borrow_rate_0: 10,
                borrow_rate_1: 20,
                borrow_rate_2: 30,
                borrow_rate_3: 40,
                utilization_rate_1: 10,
                utilization_rate_2: 20,
                management_fee_rate: 10,
                flags: PoolFlags::ALLOW_LENDING.bits(),
                reserved: 0,
            },
            oracle,
        };
        ctx.margin.create_pool(&pool_info).await?;
        ctx.tokens
            .set_price(
                &mint,
                &TokenPrice {
                    exponent: -8,
                    price: 100_000_000,
                    confidence: 1_000_000,
                    twap: 100_000_000,
                },
            )
            .await?;
        let vault = ctx
            .tokens
            .create_account_funded(&mint, &proctor.pubkey(), 100_000_000)
            .await?;
        tokens.push(Asset {
            mint,
            oracle,
            vault,
        });
    }
    // create swap pools
    let pairs = tokens
        .iter()
        .combinations(2)
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect::<Vec<(Asset, Asset)>>();
    let mut swaps: Vec<SwapPool> = Vec::new();
    let mut mint_to_mint_to_swap: HashMap<Pubkey, HashMap<Pubkey, (SwapIdx, SwapPool)>> = HashMap::new();
    for pair in pairs.iter() {
        // Create a swap pool with sufficient liquidity
        let pool = SwapPool::configure(
            &ctx.rpc,
            &orca_swap_v2::id(),
            &pair.0.mint,
            &pair.1.mint,
            100_000_000,
            100_000_000,
        )
        .await?;
        swaps.push(pool.clone());
        mint_to_mint_to_swap.entry(pair.0.mint).or_default().insert(pair.1.mint, (SwapIdx::A, pool.clone()));
        mint_to_mint_to_swap.entry(pair.1.mint).or_default().insert(pair.0.mint, (SwapIdx::B, pool));
    }
    // create deposits
    for (
        user,
        Asset {
            mint,
            oracle: _,
            vault: _,
        },
    ) in users.iter().zip(tokens.iter().cycle())
    {
        let faucet = ctx
            .tokens
            .create_account_funded(&mint, &user.owner(), 100)
            .await?;
        user.deposit(
            &mint,
            &faucet,
            TokenChange {
                kind: ChangeKind::SetTo,
                tokens: 100,
            },
        )
        .await?;
    }
    // create loans
    tokens.rotate_right(scenario.mints / 2);
    for (
        user,
        Asset {
            mint,
            oracle: _,
            vault: _,
        },
    ) in users.iter().zip(tokens.iter().cycle())
    {
        user.borrow(
            &mint,
            TokenChange {
                kind: ChangeKind::SetTo,
                tokens: 10,
            },
        )
        .await?;
        let sink = ctx.tokens.create_account(&mint, &user.owner()).await?;
        user.withdraw(
            &mint,
            &sink,
            TokenChange {
                kind: ChangeKind::SetTo,
                tokens: 10,
            },
        )
        .await?;
    }
    // incrementally lower prices of half of the assets
    let mut price = 100_000_000.0;
    let assets_to_devalue = tokens[0..tokens.len() / 2].to_vec();
    let pubkeys_to_devalue = assets_to_devalue.iter().map(|a|a.mint).collect::<HashSet<Pubkey>>();
    let mut devalue_to_pair: HashMap<Pubkey, (Pubkey, Pubkey)> = HashMap::new();
    for pair in pairs.iter() {
        if pubkeys_to_devalue.contains(&pair.0.mint) && !pubkeys_to_devalue.contains(&pair.1.mint) {
            devalue_to_pair.insert(pair.0.mint, (pair.0.mint, pair.1.mint));
        }
        if pubkeys_to_devalue.contains(&pair.1.mint) && !pubkeys_to_devalue.contains(&pair.0.mint) {
            devalue_to_pair.insert(pair.1.mint, (pair.0.mint, pair.1.mint));
        }
    }
    for _ in 0..100 {
        price *= 0.99;
        for Asset {
            mint,
            oracle: _,
            vault,
        } in assets_to_devalue.iter()
        {
            ctx.tokens
                .set_price(
                    &mint,
                    &TokenPrice {
                        exponent: -8,
                        price: price as i64,
                        confidence: 1_000_000,
                        twap: 100_000_000,
                    },
                )
                .await?;
            for (one, two) in devalue_to_pair.get(mint) {
                if one == mint {
                    
                }
                if two == mint {
                    
                }
            }
        }
    }

    Ok(())
}

/// Returns the amount of asset a to swap into (or out for negative) the pool to slip
/// the asset to its desired price relative to the other asset
fn set_constant_product_price(balance_a: u64, balance_b: u64, desired_price_of_a: f64) -> i64 {
    let product = balance_a as u128 * balance_b as u128;
    (product as f64 / desired_price_of_a).sqrt() as i64 //amount of a to move into pool
}

// async fn create_many_accounts(ctx: &MarginTestContext, n: u64) -> Result<Vec<MarginUser>, anyhow::Error>{
// 	let users = Vec::new();
// 	for _ in 0..n {
// 		let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
// 		users.push(ctx.margin.user(&wallet_a, 0).await?);
// 	}

// 	Ok(users)
// }

#[tokio::test(flavor = "multi_thread")]
async fn test() {
    load_test(Scenario {
        users: 40,
        mints: 4,
    })
    .await
    .unwrap()
}

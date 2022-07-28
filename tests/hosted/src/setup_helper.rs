use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::{Error, Result};

use jet_margin_sdk::tokens::TokenPrice;
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;

use crate::{
    context::{test_context, MarginTestContext},
    margin::{MarginPoolSetupInfo, MarginUser},
};

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

pub struct TestEnvironment<'a> {
    pub mints: Vec<Pubkey>,
    pub users: Vec<TestUser<'a>>,
    pub liquidator: Keypair,
}

/// A MarginUser that takes some extra liberties
#[derive(Clone)]
pub struct TestUser<'a> {
    pub ctx: &'a MarginTestContext,
    pub user: MarginUser,
    pub liquidator: MarginUser,
    /// user's external wallet of actual assets
    pub mint_to_token_account: HashMap<Pubkey, Pubkey>,
}

impl<'a> std::fmt::Debug for TestUser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestUser")
            .field("user", &self.user.address())
            .field("liquidator", &self.liquidator.address())
            .field("mint_to_token_account", &self.mint_to_token_account)
            .finish()
    }
}

pub async fn setup_token(
    ctx: &MarginTestContext,
    decimals: u8,
    collateral_weight: u16,
    leverage_max: u16,
    price: f64,
) -> Result<Pubkey, Error> {
    let token = ctx.tokens.create_token(decimals, None, None).await?;
    let token_oracle = ctx.tokens.create_oracle(&token).await?;

    ctx.margin
        .create_pool(&MarginPoolSetupInfo {
            token,
            collateral_weight,
            max_leverage: leverage_max,
            token_kind: TokenKind::Collateral,
            config: DEFAULT_POOL_CONFIG,
            oracle: token_oracle,
        })
        .await?;

    // set price to $1
    ctx.tokens
        .set_price(
            &token,
            &TokenPrice {
                exponent: -8,
                price: (price * 100_000_000.0) as i64,
                confidence: 1_000_000,
                twap: 100_000_000,
            },
        )
        .await?;

    Ok(token)
}

pub async fn create_users<'a, const COUNT: usize>(
    ctx: &'a MarginTestContext,
    liquidator_wallet: &Keypair,
) -> Result<[TestUser<'a>; COUNT]> {
    let mut users: Vec<TestUser<'a>> = vec![];
    for _ in 0..COUNT {
        users.push(setup_user(ctx, liquidator_wallet, vec![]).await?);
    }

    Ok(users.try_into().unwrap())
}

/// (token_mint, balance in wallet, balance in pools)
pub async fn setup_user<'a>(
    ctx: &'a MarginTestContext,
    liquidator_wallet: &Keypair,
    tokens: Vec<(Pubkey, u64, u64)>,
) -> Result<TestUser<'a>> {
    // Create our two user wallets, with some SOL funding to get started
    let wallet = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user = ctx.margin.user(&wallet, 0).await?;
    let user_liq = ctx
        .margin
        .liquidator(liquidator_wallet, &wallet.pubkey(), 0)
        .await?;

    // Initialize the margin accounts for each user
    user.create_account().await?;

    let mut mint_to_token_account = HashMap::new();
    for (mint, in_wallet, in_pool) in tokens {
        // Create some tokens for each user to deposit
        let token_account = ctx
            .tokens
            .create_account_funded(&mint, &wallet.pubkey(), in_wallet + in_pool)
            .await?;
        mint_to_token_account.insert(mint, token_account);

        if in_pool > 0 {
            // Deposit user funds into their margin accounts
            user.deposit(&mint, &token_account, TokenChange::shift(in_pool))
                .await?;
        }

        // Verify user tokens have been deposited
        assert_eq!(in_wallet, ctx.tokens.get_balance(&token_account).await?);
    }

    user.refresh_all_pool_positions().await?;

    Ok(TestUser {
        ctx,
        user,
        liquidator: user_liq,
        mint_to_token_account,
    })
}

/// Environment where no user has a balance
pub async fn build_environment_with_no_balances<'a>(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment<'a>), Error> {
    let ctx = test_context().await;
    let liquidator = ctx.create_liquidator(100).await?;
    let mut mints: Vec<Pubkey> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1.0).await?;
        mints.push(mint);
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, &liquidator, vec![]).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            liquidator,
        },
    ))
}

/// Environment where every user has 100 of every token in their wallet but no pool deposits
pub async fn build_environment_with_raw_token_balances<'a>(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment<'a>), Error> {
    let ctx = test_context().await;
    let liquidator = ctx.create_liquidator(100).await?;
    let mut mints: Vec<Pubkey> = Vec::new();
    let mut wallets: Vec<(Pubkey, u64, u64)> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1.0).await?;
        mints.push(mint);
        wallets.push((mint, 100, 0));
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, &liquidator, wallets.clone()).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            liquidator,
        },
    ))
}


// pub struct SetupUser< {
//     existing_user: Option<TestUser>,
//     actions: Vec<PoolAction>,
// }

// #[derive(Clone, Copy)]
// pub struct PoolAction {
//     mint: Pubkey,
//     amount: u64,
//     direction: PoolDirection,
// }

// #[derive(Clone, Copy)]
// pub enum PoolDirection {
//     Mint,
//     Deposit,
//     Borrow,
// }

// pub async fn setup_user_with_actions(
//     ctx: &MarginTestContext,
//     liquidator: Keypair,
//     user_setup: SetupUser,
// ) -> Result<TestUser> {
//     let SetupUser {
//         existing_user,
//         actions,
//     } = user_setup;
//     let mut user = match existing_user {
//         Some(user) => user.clone(),
//         None => setup_user(ctx, &liquidator, vec![]).await?,
//     };
//     for action in actions {
//         user.act(action);
//     }

//     Ok(user)
// }

// struct TestOrchestrator<'a> {
//     ctx: &'a MarginTestContext,
//     users: HashMap<Pubkey, TestUser<'a>>,
//     tokens: HashMap<Pubkey, TestToken>,
// }


// impl<'a> TestOrchestrator<'a> {
//     fn set_price() {

//     }
// }

// struct TestToken {
//     mint: Pubkey,
//     last_updated: UnixTimestamp,
//     latest_price: TokenPrice,
// }

// struct TokenPool {

// }

// impl<'a> TestUser<'a> {
//     pub async fn token_account(&mut self, mint: &Pubkey) -> Result<Pubkey> {
//         let token_account = match self.mint_to_token_account.entry(*mint) {
//             Entry::Occupied(entry) => entry.get().clone(),
//             Entry::Vacant(entry) => *entry.insert(
//                 self.ctx
//                     .tokens
//                     .create_account(&mint, &self.user.owner())
//                     .await?,
//             ),
//         };

//         Ok(token_account)
//     }

//     pub async fn mint(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
//         let token_account = self.token_account(mint).await?;
//         self.ctx.tokens.mint(mint, &token_account, amount).await
//     }

//     pub async fn deposit(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
//         let token_account = self.token_account(mint).await?;
//         self.user
//             .deposit(mint, &token_account, TokenChange::shift(amount))
//             .await
//     }

//     pub async fn borrow(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
//         self.ctx.tokens.refresh_to_same_price(mint).await?;
//         self.user
//             .borrow(mint, TokenChange::shift(amount))
//             .await
//     }

//     pub async fn withdraw(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
//         let token_account = self.token_account(mint).await?;
//         self.user
//             .withdraw(mint, &token_account, TokenChange::shift(amount))
//             .await
//     }
// }

pub async fn setup_token2s(
    ctx: &MarginTestContext,
    decimals: u8,
    collateral_weight: u16,
    leverage_max: u16,
    price: i64,
) -> Result<Pubkey, Error> {
    let token = ctx.tokens.create_token(decimals, None, None).await?;
    let token_oracle = ctx.tokens.create_oracle(&token).await?;

    ctx.margin
        .create_pool(&MarginPoolSetupInfo {
            token,
            collateral_weight,
            max_leverage: leverage_max,
            token_kind: TokenKind::Collateral,
            config: DEFAULT_POOL_CONFIG,
            oracle: token_oracle,
        })
        .await?;

    // set price to $1
    ctx.tokens
        .set_price(
            &token,
            &TokenPrice {
                exponent: -8,
                price: price * 100_000_000_i64,
                confidence: 1_000_000,
                twap: 100_000_000,
            },
        )
        .await?;

    Ok(token)
}

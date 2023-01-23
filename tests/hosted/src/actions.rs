use jet_client::fixed_term::{util::rate_to_price, MarketInfo};
use jet_instructions::fixed_term::OrderParams;
use jet_margin_sdk::fixed_term::event_consumer::EventConsumer;
use solana_sdk::{clock::Clock, pubkey::Pubkey};

use jet_client_native::{JetSimulationClient, JetSimulationClientResult, SimulationClient};

use crate::context::TestContext;

pub type MarginAccountClient = jet_client::margin::MarginAccountClient<SimulationClient>;

pub struct Token {
    pub mint: Pubkey,
    pub decimals: u8,
}

impl Token {
    pub fn from_client(client: &JetSimulationClient, mint: Pubkey) -> Self {
        Self {
            mint,
            decimals: client.state().token_info(&mint).unwrap().decimals,
        }
    }

    pub fn from_context(ctx: &TestContext, name: &str) -> Self {
        let actual_name = format!("{}-{name}", &ctx.config.airspaces[0].name);

        ctx.config
            .tokens
            .iter()
            .find(|t| t.name == actual_name)
            .map(|t| Self {
                mint: t.mint,
                decimals: t.decimals,
            })
            .unwrap()
    }

    pub fn amount(&self, value: f64) -> u64 {
        amount_from_f64(self, value)
    }
}

pub async fn add_time(ctx: &TestContext, increment: i64) {
    let current = ctx.rpc().get_clock().await.unwrap();

    ctx.rpc()
        .set_clock(Clock {
            unix_timestamp: current.unix_timestamp + increment,
            ..current
        })
        .await
        .unwrap()
}

/// change price of a token
pub async fn set_price(ctx: &TestContext, token: &Token, price: f64, confidence: f64) {
    ctx.set_price(&token.mint, price, confidence).await.unwrap()
}

/// airdrop tokens to a user client
pub async fn airdrop(user: &JetSimulationClient, token: &Token, amount: u64) {
    user.test_service()
        .token_request(&token.mint, amount)
        .await
        .unwrap();

    user.state().sync_all().await.unwrap();
}

/// sync all user client states
pub async fn sync_all(users: &[JetSimulationClient]) {
    for user in users {
        user.state().sync_all().await.unwrap();
    }
}

pub async fn deposit(
    account: &MarginAccountClient,
    token: &Token,
    amount: u64,
) -> JetSimulationClientResult<()> {
    account.deposit(&token.mint, amount, None).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn withdraw(
    account: &MarginAccountClient,
    token: &Token,
    amount: u64,
) -> JetSimulationClientResult<()> {
    account.withdraw(&token.mint, amount, None).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn pool_lend(
    account: &MarginAccountClient,
    token: &Token,
    amount: u64,
) -> JetSimulationClientResult<()> {
    account.pool(&token.mint).lend(amount).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn pool_borrow(
    account: &MarginAccountClient,
    token: &Token,
    amount: u64,
) -> JetSimulationClientResult<()> {
    account.pool(&token.mint).borrow(amount, None).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn pool_repay(
    account: &MarginAccountClient,
    token: &Token,
    amount: Option<u64>,
) -> JetSimulationClientResult<()> {
    account.pool(&token.mint).repay(amount).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn pool_withdraw(
    account: &MarginAccountClient,
    token: &Token,
    amount: Option<u64>,
) -> JetSimulationClientResult<()> {
    account.pool(&token.mint).withdraw(amount, None).await?;
    account.sync().await.unwrap();

    Ok(())
}

pub async fn offer_loan(
    account: &MarginAccountClient,
    market: &MarketInfo,
    amount: u64,
    interest_rate: f64,
) -> JetSimulationClientResult<()> {
    let rate = f64_rate_to_bps(interest_rate);

    account
        .fixed_term(&market.address)
        .unwrap()
        .offer_loan(amount, rate)
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();
    Ok(())
}

pub async fn offer_loan_no_auto_stake(
    account: &MarginAccountClient,
    market: &MarketInfo,
    amount: u64,
    interest_rate: f64,
) -> JetSimulationClientResult<()> {
    let rate = f64_rate_to_bps(interest_rate);
    let params = OrderParams {
        max_ticket_qty: u64::MAX,
        max_underlying_token_qty: amount,
        limit_price: rate_to_price(rate as u64, market.borrow_tenor),
        match_limit: u64::MAX,
        post_only: false,
        post_allowed: true,
        auto_stake: false,
        auto_roll: false,
    };

    account
        .fixed_term(&market.address)
        .unwrap()
        .offer_loan_with_params(params)
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();
    Ok(())
}

pub async fn request_loan(
    account: &MarginAccountClient,
    market: &MarketInfo,
    amount: u64,
    interest_rate: f64,
) -> JetSimulationClientResult<()> {
    let rate = f64_rate_to_bps(interest_rate);

    account
        .fixed_term(&market.address)
        .unwrap()
        .request_loan(amount, rate)
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();
    Ok(())
}

pub async fn cancel_order(
    account: &MarginAccountClient,
    market: &MarketInfo,
    order_id: u128,
) -> JetSimulationClientResult<()> {
    account
        .fixed_term(&market.address)
        .unwrap()
        .cancel_order(order_id)
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();
    Ok(())
}

pub async fn sell_tickets(
    account: &MarginAccountClient,
    market: &MarketInfo,
    amount: u64,
    limit_price: f64,
) -> JetSimulationClientResult<()> {
    account
        .fixed_term(&market.address)
        .unwrap()
        .sell_tickets(amount, limit_price)
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();
    Ok(())
}

pub async fn repay_term_loan(
    account: &MarginAccountClient,
    market: &MarketInfo,
    max_amount: u64,
) -> JetSimulationClientResult<()> {
    account
        .fixed_term(&market.address)
        .unwrap()
        .repay(max_amount)
        .await?;

    account.sync().await.unwrap();
    Ok(())
}

pub async fn market_settle(
    account: &MarginAccountClient,
    market: &MarketInfo,
) -> JetSimulationClientResult<()> {
    account
        .fixed_term(&market.address)
        .unwrap()
        .settle()
        .await?;

    account.sync().await.unwrap();
    Ok(())
}

pub async fn redeem_term_deposits(
    account: &MarginAccountClient,
    market: &MarketInfo,
) -> JetSimulationClientResult<()> {
    account
        .fixed_term(&market.address)
        .unwrap()
        .redeem_deposits()
        .await?;

    account.sync().await.unwrap();
    jet_client::state::fixed_term::sync(account.client().state())
        .await
        .unwrap();

    Ok(())
}

pub async fn consume_events(ctx: &TestContext, market: &MarketInfo) {
    let consumer = EventConsumer::new(ctx.rpc().clone());

    consumer.load_markets(&[market.address]).await.unwrap();
    consumer.sync_users().await.unwrap();
    consumer.sync_queues().await.unwrap();

    while consumer.pending_events(&market.address).unwrap() > 0 {
        consumer.consume().await.unwrap();
    }
}

pub fn position_balance(account: &MarginAccountClient, token: &Token) -> u64 {
    account
        .positions()
        .iter()
        .find(|p| p.token == token.mint)
        .unwrap()
        .balance
}

pub fn wallet_balance(user: &JetSimulationClient, token: &Token) -> u64 {
    user.wallet_balance(&token.mint)
}

pub fn amount_from_f64(token: &Token, amount: f64) -> u64 {
    let exponent = token.decimals as u32;
    let one = 10i64.pow(exponent) as f64;

    (one * amount).round() as u64
}

pub fn amount_to_f64(token: &Token, amount: u64) -> f64 {
    let exponent = token.decimals as u32;
    let one = 10i64.pow(exponent) as f64;

    amount as f64 / one
}

fn f64_rate_to_bps(f: f64) -> u32 {
    let bps = f * 100.0;
    assert!(bps <= u32::MAX as f64);
    assert!(bps >= 0.0);
    bps.round() as u32
}

use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use jet_fixed_term::{
    control::state::Market,
    margin::{origination_fee::FEE_UNIT, state::TermLoan},
    orderbook::state::OrderParams,
    tickets::state::TermDeposit,
};
use jet_instructions::fixed_term::{derive_term_deposit, FixedTermIxBuilder};

use crate::{
    bail,
    client::{ClientResult, ClientState},
    margin::MarginAccountClient,
    state::fixed_term::{MarketState, OrderEntry, UserState},
    NetworkUserInterface,
};

use self::interest_pricing::f64_to_fp32;

mod interest_pricing;
pub mod util;

/// Details about a fixed term market
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MarketInfo {
    /// The address of the market
    pub address: Pubkey,

    /// The airspace the market is a part of
    pub airspace: Pubkey,

    /// The address of the token mint for the underlying asset being loaned in the market
    pub token: Pubkey,

    /// The address of the token mint for the tickets that can be redeemed
    pub ticket: Pubkey,

    /// Duration of a loan for borrowers (in seconds)
    pub borrow_tenor: u64,

    /// Duration of a loan for lenders
    pub lend_tenor: u64,

    /// The fee applied onto new loans
    pub origination_fee: f64,
}

/// Client for interacting with the margin program
#[derive(Clone)]
pub struct FixedTermMarketClient<I> {
    client: Arc<ClientState<I>>,
}

impl<I: NetworkUserInterface> FixedTermMarketClient<I> {
    pub(crate) fn new(inner: Arc<ClientState<I>>) -> Self {
        Self { client: inner }
    }

    /// Sync all data for fixed term markets
    pub async fn sync(&self) -> ClientResult<I, ()> {
        crate::state::fixed_term::sync(self.client.state()).await
    }

    /// Get the set of loaded markets
    pub fn markets(&self) -> Vec<MarketInfo> {
        let mut result = vec![];

        self.client
            .state()
            .for_each(|address, state: &MarketState| {
                let origination_fee = (state.market.origination_fee as f64) / (FEE_UNIT as f64);

                result.push(MarketInfo {
                    address: *address,
                    airspace: state.market.airspace,
                    token: state.market.underlying_token_mint,
                    ticket: state.market.ticket_mint,
                    borrow_tenor: state.market.borrow_tenor,
                    lend_tenor: state.market.lend_tenor,
                    origination_fee,
                })
            });

        result
    }
}

/// Client for interacting with a fixed term market, from the perspective of a margin account
#[derive(Clone)]
pub struct MarginAccountMarketClient<I> {
    pub(crate) client: Arc<ClientState<I>>,
    pub(crate) builder: FixedTermIxBuilder,
    pub(crate) account: MarginAccountClient<I>,
    pub(crate) market: Market,
}

impl<I: NetworkUserInterface> MarginAccountMarketClient<I> {
    pub fn from_address(
        account: MarginAccountClient<I>,
        market_address: &Pubkey,
    ) -> ClientResult<I, Self> {
        let state = match account.client.state().get::<MarketState>(market_address) {
            Some(m) => m,
            None => {
                bail!("attempting to create market client for unknown/unloaded market {market_address}")
            }
        };

        let builder = FixedTermIxBuilder::new_from_state(account.client.signer(), &state.market);

        Ok(Self {
            client: account.client.clone(),
            account,
            builder,
            market: state.market,
        })
    }

    /// Get the current outstanding loans in this market for the current user
    pub fn loans(&self) -> Vec<Arc<TermLoan>> {
        self.get_user_market_state()
            .map(|state| state.loans().into_iter().collect())
            .unwrap_or_default()
    }

    /// Get the set of deposits that this user can eventually withdraw at maturity
    pub fn deposits(&self) -> Vec<Arc<TermDeposit>> {
        self.get_user_market_state()
            .map(|state| state.deposits().into_iter().collect())
            .unwrap_or_default()
    }

    /// Get the set of pending orders placed in the market by the current user
    pub fn orders(&self) -> Vec<OrderEntry> {
        let market = self.get_market_state();
        let asks = market
            .asks
            .iter()
            .filter(|entry| entry.owner == self.account.address);
        let bids = market
            .bids
            .iter()
            .filter(|entry| entry.owner == self.account.address);

        asks.chain(bids).cloned().collect()
    }

    /// Sync the user market state
    pub async fn sync(&self) -> ClientResult<I, ()> {
        crate::state::fixed_term::sync_user_accounts(self.client.state()).await
    }

    /// Place an order to lend tokens in the market
    ///
    /// # Parameters
    ///
    /// * `amount` - The amount of tokens to offer for lending
    /// * `interest_rate` - The interest rate to lend the tokens at (in basis points)
    pub async fn offer_loan(&self, amount: u64, interest_rate: u32) -> ClientResult<I, ()> {
        let params = OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: amount,
            limit_price: self.limit_price_for_rate(interest_rate),
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: self.should_auto_roll_lend_order(),
        };

        self.offer_loan_with_params(params).await
    }

    /// Place an order to borrow tokens in the market
    ///
    /// # Parameters
    ///
    /// * `amount` - The desired amount of tokens to borrow
    /// * `interest_rate` - The interest rate to borrow the tokens at (in basis points)
    pub async fn request_loan(&self, amount: u64, interest_rate: u32) -> ClientResult<I, ()> {
        let params = OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: amount,
            limit_price: self.limit_price_for_rate(interest_rate),
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: self.should_auto_roll_borrow_order(),
        };

        self.request_loan_with_params(params).await
    }

    /// Place an order to sell tickets in the market
    ///
    /// # Parameters
    ///
    /// * `amount` - The desired amount of tickets to be sold
    /// * `price` - The price to sell the tickets at
    pub async fn sell_tickets(&self, amount: u64, price: f64) -> ClientResult<I, ()> {
        let params = OrderParams {
            max_ticket_qty: amount,
            max_underlying_token_qty: u64::MAX,
            limit_price: f64_to_fp32(price),
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: false,
        };

        self.sell_tickets_with_params(params).await
    }

    /// Lend tokens in the market, immediately matching any borrow orders at the best
    /// available interest rate.
    ///
    /// # Parameters
    ///
    /// * `amount` - The amount of tokens to offer for lending
    pub async fn lend_now(&self, amount: u64) -> ClientResult<I, ()> {
        let params = OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: amount,
            limit_price: u32::MAX as u64,
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: false,
            auto_stake: true,
            auto_roll: self.should_auto_roll_lend_order(),
        };

        self.offer_loan_with_params(params).await
    }

    /// Borrow tokens in the market, immediately matching any lend orders at the best
    /// available interest rate.
    ///
    /// # Parameters
    ///
    /// * `amount` - The amount of tokens to request to borrow
    pub async fn borrow_now(&self, amount: u64) -> ClientResult<I, ()> {
        let params = OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: amount,
            limit_price: 1,
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: false,
            auto_stake: true,
            auto_roll: self.should_auto_roll_borrow_order(),
        };

        self.request_loan_with_params(params).await
    }

    /// Pay back outstanding loans, up to an amount specified.
    ///
    /// This will start with the oldest loan first to be repaid, and continue repaying
    /// newer loans until the specifed maximum is reached.
    ///
    /// # Parameters
    ///
    /// * `max_repayment` - The upper limit of tokens to transfer as repayment for loans.
    pub async fn repay(&self, max_repayment: u64) -> ClientResult<I, ()> {
        let Some(user_state) = self.get_user_market_state() else {
            return Ok(());
        };

        let mut ixns = vec![];
        let mut repay_remain = max_repayment;

        for loan in user_state.loans() {
            let to_repay = std::cmp::min(repay_remain, loan.balance);
            repay_remain -= to_repay;

            let source_account =
                get_associated_token_address(&self.account.address, &self.builder.token_mint());

            ixns.push(
                self.account
                    .builder
                    .adapter_invoke(self.builder.margin_repay(
                        &self.account.address,
                        &loan.payer,
                        &self.account.address,
                        &source_account,
                        loan.sequence_number,
                        to_repay,
                    )),
            );
        }

        self.client.send(&ixns).await
    }

    /// Cancel a previous request to lend or borrow
    ///
    /// # Parameters
    ///
    /// `order_id` - The ID for the order to be canceled
    pub async fn cancel_order(&self, order_id: u128) -> ClientResult<I, ()> {
        let ixns = vec![self
            .account
            .builder
            .adapter_invoke(self.builder.cancel_order(self.account.address(), order_id))];

        self.client.send(&ixns).await
    }

    /// Redeem any matured deposits belonging to this account
    pub async fn redeem_deposits(&self) -> ClientResult<I, ()> {
        let current_time = chrono::Utc::now().timestamp();
        let matured_deposits = self
            .deposits()
            .into_iter()
            .filter(|d| d.matures_at <= current_time)
            .collect::<Vec<_>>();

        if matured_deposits.is_empty() {
            log::debug!("no mature deposits for user {}", self.account.address);
            return Ok(());
        }

        let mut ixns = vec![];

        let token_account = self
            .account
            .with_deposit_position(&self.builder.token_mint(), &mut ixns)
            .await?;

        ixns.extend(matured_deposits.into_iter().map(|d| {
            let user_key = self.builder.margin_user_account(self.account.address());
            let deposit_key =
                derive_term_deposit(&self.builder.market(), &user_key, d.sequence_number);

            self.account
                .builder
                .adapter_invoke(self.builder.margin_redeem_deposit(
                    self.account.address(),
                    deposit_key,
                    Some(token_account),
                ))
        }));

        self.client.send(&ixns).await
    }

    /// Settle tokens from matched orders
    pub async fn settle(&self) -> ClientResult<I, ()> {
        let mut ixns = vec![];

        ixns.push(
            self.account
                .builder
                .adapter_invoke(self.builder.settle(self.account.address)),
        );

        self.client.send(&ixns).await
    }

    /// Place an order to lend in the market
    ///
    /// # Parameters
    ///
    /// * `params` - The order parameters
    pub async fn offer_loan_with_params(&self, params: OrderParams) -> ClientResult<I, ()> {
        let deposit_account =
            get_associated_token_address(&self.account.address, &self.builder.token_mint());

        let mut ixns = vec![];

        self.with_user_registration(&mut ixns).await?;

        ixns.push(
            self.account
                .builder
                .adapter_invoke(self.builder.margin_lend_order(
                    self.account.address,
                    Some(deposit_account),
                    params,
                    self.get_next_deposit_seq_no(),
                )),
        );

        self.account.send_with_refresh(&ixns).await
    }

    /// Place an order to borrow in the market
    ///
    /// # Parameters
    ///
    /// * `params` - The order parameters
    pub async fn request_loan_with_params(&self, params: OrderParams) -> ClientResult<I, ()> {
        let mut ixns = vec![];

        let token_account = self
            .account
            .with_deposit_position(&self.builder.token_mint(), &mut ixns)
            .await?;

        self.with_user_registration(&mut ixns).await?;

        ixns.push(
            self.account
                .builder
                .adapter_invoke(self.builder.margin_borrow_order(
                    self.account.address,
                    Some(token_account),
                    params,
                    self.get_next_loan_seq_no(),
                )),
        );

        self.account.send_with_refresh(&ixns).await
    }

    /// Place an order to sell tickets in the market
    ///
    /// # Parameters
    ///
    /// * `params` - The order parameters
    pub async fn sell_tickets_with_params(&self, params: OrderParams) -> ClientResult<I, ()> {
        let mut ixns = vec![];

        let ticket_account =
            get_associated_token_address(&self.account.address, &self.builder.ticket_mint());

        let token_account = self
            .account
            .with_deposit_position(&self.builder.token_mint(), &mut ixns)
            .await?;

        self.with_user_registration(&mut ixns).await?;

        ixns.push(
            self.account
                .builder
                .adapter_invoke(self.builder.margin_sell_tickets_order(
                    self.account.address,
                    Some(ticket_account),
                    Some(token_account),
                    params,
                )),
        );

        self.account.send_with_refresh(&ixns).await
    }

    async fn with_user_registration(&self, ixns: &mut Vec<Instruction>) -> ClientResult<I, ()> {
        let user_market_account = self.builder.margin_user_account(self.account.address);

        self.account
            .with_deposit_position(&self.builder.ticket_mint(), ixns)
            .await?;

        if !self.client.account_exists(&user_market_account).await? {
            ixns.push(
                self.account
                    .builder
                    .adapter_invoke(self.builder.initialize_margin_user(self.account.address)),
            );

            ixns.push(
                self.account
                    .builder
                    .accounting_invoke(self.builder.refresh_position(self.account.address, true)),
            )
        }

        Ok(())
    }

    fn should_auto_roll_lend_order(&self) -> bool {
        self.get_user_market_state()
            .map(|s| s.borrow_roll_config.limit_price > 0)
            .unwrap_or_default()
    }

    fn should_auto_roll_borrow_order(&self) -> bool {
        self.get_user_market_state()
            .map(|s| s.borrow_roll_config.limit_price > 0)
            .unwrap_or_default()
    }

    fn get_next_loan_seq_no(&self) -> u64 {
        let user_account = self.get_user_market_state();
        user_account
            .map(|u| u.debt.next_new_loan_seqno())
            .unwrap_or_default()
    }

    fn get_next_deposit_seq_no(&self) -> u64 {
        let user_account = self.get_user_market_state();
        user_account
            .map(|u| u.assets.next_new_deposit_seqno())
            .unwrap_or_default()
    }

    fn get_user_market_state(&self) -> Option<Arc<UserState>> {
        let address = self.builder.margin_user_account(self.account.address);
        self.client.state().get(&address)
    }

    fn get_market_state(&self) -> Arc<MarketState> {
        self.client.state().get(&self.builder.market()).unwrap()
    }

    fn limit_price_for_rate(&self, interest_rate: u32) -> u64 {
        util::rate_to_price(interest_rate as u64, self.market.borrow_tenor)
    }
}

pub(crate) fn instruction_for_refresh<I: NetworkUserInterface>(
    account: &MarginAccountClient<I>,
    token: &Pubkey,
    refreshing_tokens: &mut HashSet<Pubkey>,
) -> ClientResult<I, Instruction> {
    let found = account.client.state().filter(|_, state: &MarketState| {
        state.market.claims_mint == *token || state.market.ticket_collateral_mint == *token
    });

    let Some((_, market_state)) = found.into_iter().next() else {
        bail!(
            "account {} contains fixed-term token {} belonging to unknown market",
            account.address, token
        );
    };

    refreshing_tokens.insert(market_state.market.claims_mint);
    refreshing_tokens.insert(market_state.market.ticket_collateral_mint);

    let builder = FixedTermIxBuilder::new_from_state(account.client.signer(), &market_state.market);

    Ok(account
        .builder
        .accounting_invoke(builder.refresh_position(account.address, true)))
}

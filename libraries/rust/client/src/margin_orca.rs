use std::{collections::HashSet, sync::Arc};

use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use jet_solana_client::{rpc::SolanaRpcExtra, transaction::TransactionBuilder};
use num_traits::pow::Pow;
use orca_whirlpool::{
    math::{mul_u256, tick_index_from_sqrt_price, U256Muldiv},
    state::Position as WhirlpoolPosition,
};
use rust_decimal::{prelude::*, Decimal, MathematicalOps};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
};

use jet_instructions::{
    margin_orca::{derive, MarginOrcaIxBuilder},
    orca::{derive_tick_array, start_tick_index},
};
use jet_instructions::{
    margin_orca::{WhirlpoolPositionSummary, WhirlpoolSummary},
    orca::ORCA_WHIRLPOOL_PROGRAM,
};

use crate::{
    bail,
    client::{ClientResult, ClientState},
    margin::MarginAccountClient,
    state::margin_orca::{UserState, WhirlpoolConfigState},
};

/// Client for interacting with the margin program
#[derive(Clone)]
pub struct MarginOrcaClient {
    client: Arc<ClientState>,
}

impl MarginOrcaClient {
    // pub(crate) fn new(inner: Arc<ClientState>) -> Self {
    //     Self { client: inner }
    // }

    /// Sync all data for whirlpools
    pub async fn sync(&self) -> ClientResult<()> {
        crate::state::margin_orca::sync(self.client.state()).await
    }

    /// Get the set of loaded whirlpool configs
    pub fn configs(&self) -> Vec<WhirlpoolConfigState> {
        let mut result = vec![];

        self.client
            .state()
            .for_each(|_, state: &WhirlpoolConfigState| result.push(state.clone()));

        result
    }
}

/// Client for interacting with a whirlpool, from the perspective of a margin account
#[derive(Clone)]
pub struct MarginAccountOrcaClient {
    pub(crate) client: Arc<ClientState>,
    pub(crate) builder: MarginOrcaIxBuilder,
    pub(crate) account: MarginAccountClient,
    #[allow(unused)]
    pub(crate) state: Arc<WhirlpoolConfigState>,
    /// The whirlpool address used to create the client.
    /// If a user would like to interact with multiple whirlpools, they should
    /// create a new client for each whirlpool. This is to simplify interactions
    /// as the whirlpool address is almost always required for transacting.
    pub(crate) whirlpool: WhirlpoolSummary,
}

impl MarginAccountOrcaClient {
    pub fn from_whirlpool(account: MarginAccountClient, whirlpool: &Pubkey) -> ClientResult<Self> {
        // Find the state that has this whirlpool
        let Some((_, state)) = account.client.state().get_all::<WhirlpoolConfigState>().into_iter().find(|(_, state)| {
            state.whirlpools.contains_key(whirlpool)
        }) else {
            bail!("attempting to create whirlpool client for unknown/unloaded whirlpool {whirlpool}")
        };

        let builder = MarginOrcaIxBuilder::new_from_config(&state.config);
        let whirlpool = (
            *whirlpool,
            state.whirlpools.get(whirlpool).unwrap().as_ref(),
        )
            .into();

        Ok(Self {
            client: account.client.clone(),
            account,
            builder,
            state,
            whirlpool,
        })
    }

    /// Get the current whirlpool positions
    pub fn positions(&self) -> Vec<Arc<WhirlpoolPosition>> {
        self.get_user_position_meta_state()
            .map(|state| state.positions().values().cloned().collect())
            .unwrap_or_default()
    }

    /// Sync the user state
    pub async fn sync(&self) -> ClientResult<()> {
        crate::state::margin_orca::sync_user_positions(self.client.state()).await
    }

    pub async fn register_margin_position(&self) -> ClientResult<()> {
        let ixns = vec![self.account.builder.adapter_invoke(
            self.builder
                .register_margin_position(self.account.address, self.account.client.signer()),
        )];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    pub async fn close_position_meta(&self) -> ClientResult<()> {
        let ixns = vec![self.account.builder.adapter_invoke(
            self.builder
                .close_margin_position(self.account.address, self.account.client.signer()),
        )];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    /// Open a whirlpool position and return summary infrmation about the position
    pub async fn open_whirlpool_position(
        &self,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> ClientResult<WhirlpoolPositionSummary> {
        // Check if the ticks exist, create them if not
        let mut ixns = vec![];
        self.with_tick_array(&mut ixns, tick_lower_index).await?;
        self.with_tick_array(&mut ixns, tick_upper_index).await?;
        let (position_ix, _, position) = self.builder.open_whirlpool_position(
            self.account.address,
            self.account.client.signer(),
            self.whirlpool.address,
            tick_lower_index,
            tick_upper_index,
        );
        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();
        ixns.push(
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
        );
        ixns.push(self.account.builder.adapter_invoke(position_ix));

        let builder = TransactionBuilder {
            instructions: ixns,
            signers: vec![],
        };

        // Small enough to not need lookup tables
        self.client.send(&builder).await?;

        let position_summary = self.update_position(&position).await?;

        Ok(position_summary)
    }

    pub async fn close_whirlpool_position(&self, mint: Pubkey) -> ClientResult<()> {
        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();
        let ixns = vec![
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
            self.account
                .builder
                .adapter_invoke(self.builder.close_whirlpool_position(
                    self.account.address,
                    self.account.client.signer(),
                    mint,
                )),
        ];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    pub async fn add_liquidity(
        &mut self,
        position_summary: &WhirlpoolPositionSummary,
        token_max_a: u64,
        token_max_b: u64,
    ) -> ClientResult<()> {
        self.refresh_prices().await?;
        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();

        let min_sqrt_price = position_summary.lower_sqrt_price();
        let max_sqrt_price = position_summary.upper_sqrt_price();
        let curr_sqrt_price = self.whirlpool.current_sqrt_price;
        let current_tick_index = self.whirlpool.current_tick_index;

        // If the current price is outside of the range, the user can only add liquidity
        // on one side of the pool.
        let liquidity_amount = if current_tick_index < position_summary.tick_lower_index {
            liquidity_for_token_a(token_max_a, min_sqrt_price, max_sqrt_price)
        } else if current_tick_index > position_summary.tick_upper_index {
            liquidity_for_token_b(token_max_b, min_sqrt_price, max_sqrt_price)
        } else {
            // TODO: what is the range?
            let liquidity_a = liquidity_for_token_a(token_max_a, min_sqrt_price, curr_sqrt_price);
            let liquidity_b = liquidity_for_token_b(token_max_b, curr_sqrt_price, max_sqrt_price);
            std::cmp::min(liquidity_a, liquidity_b)
        };

        // Refresh before modifying liquidity
        let ixns = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(1_200_000),
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
            self.account
                .builder
                .adapter_invoke(self.builder.add_liquidity(
                    self.account.address,
                    &self.whirlpool,
                    position_summary,
                    liquidity_amount,
                    token_max_a,
                    token_max_b,
                )),
        ];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    pub async fn remove_liquidity(
        &mut self,
        position_summary: &WhirlpoolPositionSummary,
        token_max_a: u64,
        token_max_b: u64,
    ) -> ClientResult<()> {
        self.refresh_prices().await?;
        let position_summary = self.update_position(&position_summary.position).await?;

        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();

        let min_sqrt_price = position_summary.lower_sqrt_price();
        let max_sqrt_price = position_summary.upper_sqrt_price();
        let curr_sqrt_price = self.whirlpool.current_sqrt_price;
        let current_tick_index = self.whirlpool.current_tick_index;

        let liquidity_amount = if current_tick_index < position_summary.tick_lower_index {
            liquidity_for_token_a(token_max_a, min_sqrt_price, max_sqrt_price)
        } else if current_tick_index > position_summary.tick_upper_index {
            liquidity_for_token_b(token_max_b, min_sqrt_price, max_sqrt_price)
        } else {
            // TODO: what is the range?
            let liquidity_a = liquidity_for_token_a(token_max_a, curr_sqrt_price, max_sqrt_price);
            let liquidity_b = liquidity_for_token_b(token_max_b, min_sqrt_price, curr_sqrt_price);
            std::cmp::min(liquidity_a, liquidity_b)
        };
        let liquidity_amount = liquidity_amount.min(position_summary.liquidity);

        let ixns = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(1_200_000),
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
            self.account
                .builder
                .adapter_invoke(self.builder.remove_liquidity(
                    self.account.address,
                    &self.whirlpool,
                    &position_summary,
                    liquidity_amount,
                    token_max_a - 1, // TODO: account for slippage correctly
                    token_max_b - 1,
                )),
        ];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    pub async fn remove_all_liquidity(
        &mut self,
        position_summary: &WhirlpoolPositionSummary,
    ) -> ClientResult<()> {
        self.refresh_prices().await?;
        let position_summary = self.update_position(&position_summary.position).await?;

        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();

        let liquidity_amount = position_summary.liquidity;

        let ixns = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(1_200_000),
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
            self.account
                .builder
                .adapter_invoke(self.builder.remove_liquidity(
                    self.account.address,
                    &self.whirlpool,
                    &position_summary,
                    liquidity_amount,
                    1, // TODO: account for slippage correctly
                    1,
                )),
        ];

        // Small enough to not need lookup tables
        self.client.send(&ixns).await
    }

    /// Refresh the position by refreshing the positions in the whirlpool program and then the margin program
    pub async fn margin_refresh_position(&self) -> ClientResult<()> {
        let user_state = self.get_user_position_meta_state().unwrap();
        let (whirlpools, positions) = user_state.addresses_for_refresh();
        let ixns = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(1_200_000),
            self.account
                .builder
                .accounting_invoke(self.builder.margin_refresh_position(
                    self.account.address,
                    &whirlpools,
                    &positions,
                )),
        ];

        // Small enough to not need lookup tables
        self.account.send_with_refresh(&ixns).await
    }

    pub async fn collect_fees(
        &self,
        position_summary: &WhirlpoolPositionSummary,
    ) -> ClientResult<()> {
        let ixns = vec![self
            .account
            .builder
            .adapter_invoke(self.builder.collect_fees(
                self.account.address,
                &self.whirlpool,
                position_summary,
            ))];

        // TODO: add lookup tables
        self.account.send_with_refresh(&ixns).await
    }

    pub async fn collect_rewards(
        &self,
        position_summary: &WhirlpoolPositionSummary,
    ) -> ClientResult<()> {
        let mut ixns = vec![];

        for (index, reward) in self.whirlpool.rewards.iter().enumerate() {
            if reward.mint != Pubkey::default() {
                ixns.push(
                    self.account
                        .builder
                        .adapter_invoke(self.builder.collect_reward(
                            self.account.address,
                            &self.whirlpool,
                            position_summary,
                            reward,
                            index as u8,
                        )),
                );
            }
        }

        // TODO: add lookup tables
        self.account.send_with_refresh(&ixns).await
    }

    async fn refresh_prices(&mut self) -> ClientResult<()> {
        let whirlpool = self
            .client
            .network
            .get_anchor_account::<_>(&self.whirlpool.address)
            .await?;
        self.whirlpool = (self.whirlpool.address, &whirlpool).into();

        Ok(())
    }

    async fn update_position(&self, address: &Pubkey) -> ClientResult<WhirlpoolPositionSummary> {
        let position = self.client.network.get_anchor_account::<_>(address).await?;
        Ok(WhirlpoolPositionSummary::from_position(
            *address,
            &position,
            self.whirlpool.clone(),
        ))
    }

    async fn with_tick_array(
        &self,
        ixns: &mut Vec<Instruction>,
        tick_index: i32,
    ) -> ClientResult<()> {
        let tick_array = derive_tick_array(
            &self.whirlpool.address,
            tick_index,
            self.whirlpool.tick_spacing,
        );

        if !self.client.account_exists(&tick_array).await? {
            ixns.push(
                self.account
                    .builder
                    .adapter_invoke(self.initialize_tick_array(tick_index, tick_array)),
            );
        }

        Ok(())
    }

    fn initialize_tick_array(&self, tick_index: i32, tick_array: Pubkey) -> Instruction {
        let start_tick_index = start_tick_index(tick_index, self.whirlpool.tick_spacing, 0);
        let accounts = orca_whirlpool::accounts::InitializeTickArray {
            funder: self.client.signer(),
            whirlpool: self.whirlpool.address,
            tick_array,
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: ORCA_WHIRLPOOL_PROGRAM,
            data: orca_whirlpool::instruction::InitializeTickArray { start_tick_index }.data(),
            accounts,
        }
    }

    #[inline]
    fn get_user_position_meta_state(&self) -> Option<Arc<UserState>> {
        // TODO: calc it once
        let position_metadata =
            derive::derive_adapter_position_metadata(&self.account.address, &self.builder.address);
        self.client.state().get(&position_metadata)
    }
}

pub(crate) fn instruction_for_refresh(
    account: &MarginAccountClient,
    token: &Pubkey,
    refreshing_tokens: &mut HashSet<Pubkey>,
) -> ClientResult<Vec<Instruction>> {
    let found = account
        .client
        .state()
        .filter(|_, state: &WhirlpoolConfigState| state.config.position_mint == *token);

    let Some((config_address, state)) = found.into_iter().next() else {
        bail!(
            "account {} contains margin-orca token {} belonging to unknown token pair",
            account.address, token
        );
    };

    refreshing_tokens.insert(state.config.position_mint);

    let builder = MarginOrcaIxBuilder::new_from_config(&state.config);

    let position_meta_address =
        derive::derive_adapter_position_metadata(&account.address, &config_address);
    let user_state = account
        .client
        .state()
        .get::<UserState>(&position_meta_address)
        .unwrap();

    let whirlpools = user_state.whirlpools();

    let mut instructions = user_state
        .positions()
        .iter()
        .map(|(address, position)| {
            let whirlpool = whirlpools
                .get(&position.whirlpool)
                .expect("Position whirlpool not found");
            let whirlpool_summary = (position.whirlpool, whirlpool.as_ref());
            let position_summary =
                WhirlpoolPositionSummary::from_position(*address, position, whirlpool_summary);
            builder.update_fees_and_rewards(&position_summary)
        })
        .collect::<Vec<_>>();

    let (whirlpools, positions) = user_state.addresses_for_refresh();

    instructions.push(
        account
            .builder
            .accounting_invoke(builder.margin_refresh_position(
                account.address,
                &whirlpools,
                &positions,
            )),
    );

    Ok(instructions)
}

fn liquidity_for_token_a(amount: u64, min_sqrt_price: u128, max_sqrt_price: u128) -> u128 {
    let div = max_sqrt_price - min_sqrt_price;
    let liquidity = mul_u256(max_sqrt_price, min_sqrt_price)
        .mul(U256Muldiv::new(0, amount as u128))
        .div(U256Muldiv::new(0, div).shift_left(64), false)
        .0;

    liquidity.get_word_u128(0)
}

fn liquidity_for_token_b(amount: u64, min_sqrt_price: u128, max_sqrt_price: u128) -> u128 {
    ((amount as u128) << 64) / (max_sqrt_price - min_sqrt_price)
}

#[allow(unused)]
fn price_to_sqrt_price(price: f64, decimals_a: impl Into<i64>, decimals_b: impl Into<i64>) -> u128 {
    let c = Decimal::TEN.pow(decimals_b.into() - decimals_a.into());
    let price = Decimal::from_f64_retain(price).unwrap();
    let sqrt_price = (price * c).sqrt().unwrap() * Decimal::from(2).powi(64);

    sqrt_price.floor().to_u128().unwrap()
}

#[allow(unused)]
fn sqrt_price_to_tick_index(sqrt_price: &u128, tick_spacing: u16) -> i32 {
    let tick_index = tick_index_from_sqrt_price(sqrt_price);
    tick_index - (tick_index % tick_spacing as i32)
}

#[allow(unused)]
fn max_liquidity_amount(decimals: u8, factor: u8) -> u64 {
    let n = decimals + factor;
    10u64.pow(n as u32)
}

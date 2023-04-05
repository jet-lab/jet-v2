use num_traits::{Pow, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};

use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use orca_whirlpool::{
    math::{mul_u256, sqrt_price_from_tick_index, tick_index_from_sqrt_price, U256Muldiv},
    state::{Position, Whirlpool},
};

use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::{Account as TokenAccount, Mint};

use jet_instructions::{
    orca::{derive_position, derive_tick_array, start_tick_index, WhirlpoolIxBuilder},
    test_service::token_request,
};
use jet_solana_client::rpc::{ClientError, SolanaRpc, SolanaRpcExtra};

const PRICE_RANGE: f64 = 0.5;

/// Set the liquidity level and price for a whirlpool
pub async fn set_liquidity(
    wallet: &Keypair,
    client: &(dyn SolanaRpc + 'static),
    whirlpool_address: Pubkey,
    target_price: f64,
    liquidity_factor: u8,
) -> Result<(), ClientError> {
    let whirlpool = client
        .get_anchor_account::<Whirlpool>(&whirlpool_address)
        .await
        .unwrap();

    let mut lm = LiquidityManager {
        ix: WhirlpoolIxBuilder::from_whirlpool(wallet.pubkey(), whirlpool_address, &whirlpool),
        wallet,
        client,
        whirlpool_address,
        whirlpool: whirlpool.clone(),
    };

    // Calculate the relevant tick range based on the target price
    let token_a_mint = client
        .get_packed_account::<Mint>(&whirlpool.token_mint_a)
        .await
        .unwrap();
    let token_b_mint = client
        .get_packed_account::<Mint>(&whirlpool.token_mint_b)
        .await
        .unwrap();
    let min_price = target_price * (1.0 - PRICE_RANGE);
    let max_price = target_price * (1.0 + PRICE_RANGE);

    let min_sqrt_price =
        price_to_sqrt_price(min_price, token_a_mint.decimals, token_b_mint.decimals);
    let max_sqrt_price =
        price_to_sqrt_price(max_price, token_a_mint.decimals, token_b_mint.decimals);

    let target_sqrt_price =
        price_to_sqrt_price(target_price, token_a_mint.decimals, token_b_mint.decimals);

    // Get enough tokens to provide liquidity
    let token_a_amount = max_liquidity_amount(token_a_mint.decimals, liquidity_factor);
    let token_b_amount = max_liquidity_amount(token_b_mint.decimals, liquidity_factor);

    log::trace!("set_liquidity: tick_spacing={0} target={target_price}, min={min_price}, max={max_price}, token_a_amount={token_a_amount}, token_b_amount={token_b_amount}", whirlpool.tick_spacing);

    lm.set_wallet_liquidity(&whirlpool.token_mint_a, token_a_amount)
        .await
        .unwrap();
    lm.set_wallet_liquidity(&whirlpool.token_mint_b, token_b_amount)
        .await
        .unwrap();

    // Close any open positions for this wallet
    lm.close_positions().await.unwrap();

    // Open new position at the target price
    lm.open_position(
        min_sqrt_price,
        max_sqrt_price,
        token_a_amount,
        token_b_amount,
    )
    .await
    .unwrap();

    // Execute swaps to get to the target price
    lm.swap_to_price(target_sqrt_price).await.unwrap();

    Ok(())
}

struct LiquidityManager<'a> {
    wallet: &'a Keypair,
    client: &'a (dyn SolanaRpc + 'static),
    whirlpool_address: Pubkey,
    whirlpool: Whirlpool,
    ix: WhirlpoolIxBuilder,
}

impl<'a> LiquidityManager<'a> {
    async fn open_position(
        &self,
        min_sqrt_price: u128,
        max_sqrt_price: u128,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<(), ClientError> {
        let mut ixns = vec![];

        // Create a new mint to get an NFT for the position
        let mint_key = Keypair::new();
        let position_mint = mint_key.pubkey();

        let tick_lower_index =
            sqrt_price_to_tick_index(&min_sqrt_price, self.whirlpool.tick_spacing);
        let tick_upper_index =
            sqrt_price_to_tick_index(&max_sqrt_price, self.whirlpool.tick_spacing);

        // Open position with the calculated range
        ixns.push(
            self.ix
                .open_position(position_mint, tick_lower_index, tick_upper_index),
        );

        // Create any missing tick arrays
        self.with_tick_array(&mut ixns, tick_lower_index)
            .await
            .unwrap();
        self.with_tick_array(&mut ixns, tick_upper_index)
            .await
            .unwrap();

        // Provide tokens as liquidity to the position
        let liquidity_a = liquidity_for_token_a(token_a_amount, min_sqrt_price, max_sqrt_price);
        let liquidity_b = liquidity_for_token_b(token_b_amount, min_sqrt_price, max_sqrt_price);
        let liquidity_amount = std::cmp::min(liquidity_a, liquidity_b);

        let base_source_account =
            get_associated_token_address(&self.wallet.pubkey(), &self.whirlpool.token_mint_a);
        let quote_source_account =
            get_associated_token_address(&self.wallet.pubkey(), &self.whirlpool.token_mint_b);

        ixns.push(self.ix.increase_liquidity(
            position_mint,
            tick_lower_index,
            tick_upper_index,
            base_source_account,
            quote_source_account,
            liquidity_amount,
            token_a_amount,
            token_b_amount,
        ));

        log::trace!("increase-liquidity: liquidity={liquidity_amount}, tick_lower_index={tick_lower_index}, tick_upper_index={tick_upper_index}");

        let tx = Transaction::new_signed_with_payer(
            &ixns,
            Some(&self.wallet.pubkey()),
            &[self.wallet, &mint_key],
            self.client.get_latest_blockhash().await.unwrap(),
        );

        self.client
            .send_and_confirm_transaction_legacy(&tx)
            .await
            .unwrap();

        log::trace!("opened position {position_mint}");

        Ok(())
    }

    async fn swap_to_price(&mut self, target_sqrt_price: u128) -> Result<(), ClientError> {
        let target_tick_index = tick_index_from_sqrt_price(&target_sqrt_price);

        loop {
            let whirlpool = self
                .client
                .get_anchor_account::<Whirlpool>(&self.whirlpool_address)
                .await
                .unwrap();

            if whirlpool.tick_current_index == target_tick_index {
                break;
            }

            let token_a_amount = self.token_a_in_wallet().await.unwrap();
            let token_b_amount = self.token_b_in_wallet().await.unwrap();

            self.swap_step(target_sqrt_price, token_a_amount, token_b_amount)
                .await
                .unwrap();
        }
        Ok(())
    }

    async fn swap_step(
        &mut self,
        target_sqrt_price: u128,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<(), ClientError> {
        let (amount, a_to_b, ticks) = if self.whirlpool.sqrt_price < target_sqrt_price {
            (
                token_b_amount,
                false,
                [
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        0,
                    ),
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        1,
                    ),
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        2,
                    ),
                ],
            )
        } else {
            (
                token_a_amount,
                true,
                [
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        0,
                    ),
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        -1,
                    ),
                    start_tick_index(
                        self.whirlpool.tick_current_index,
                        self.whirlpool.tick_spacing,
                        -2,
                    ),
                ],
            )
        };

        let wallet = self.wallet.pubkey();
        let base_account = get_associated_token_address(&wallet, &self.whirlpool.token_mint_a);
        let quote_account = get_associated_token_address(&wallet, &self.whirlpool.token_mint_b);

        let mut ixns = vec![];

        for tick in &ticks {
            self.with_tick_array(&mut ixns, *tick).await.unwrap();
        }

        let max_tick_limit = ticks[2] + 32 * self.whirlpool.tick_spacing as i32;
        let sqrt_price_limit = if a_to_b {
            std::cmp::max(
                sqrt_price_from_tick_index(max_tick_limit),
                target_sqrt_price,
            )
        } else {
            std::cmp::min(
                sqrt_price_from_tick_index(max_tick_limit),
                target_sqrt_price,
            )
        };

        ixns.push(self.ix.swap(
            base_account,
            quote_account,
            ticks,
            amount,
            0,
            sqrt_price_limit,
            true,
            a_to_b,
        ));

        let tick_limit = sqrt_price_to_tick_index(&sqrt_price_limit, self.whirlpool.tick_spacing);
        log::trace!("swap step: amount={amount}, ticks={ticks:?}, limit={tick_limit}");
        self.send_transaction(&ixns).await.unwrap();

        self.whirlpool = self
            .client
            .get_anchor_account(&self.whirlpool_address)
            .await
            .unwrap();

        Ok(())
    }

    async fn close_positions(&self) -> Result<(), ClientError> {
        let wallet_tokens = self
            .client
            .get_token_accounts_by_owner(&self.wallet.pubkey())
            .await
            .unwrap();

        for (_, token_account) in wallet_tokens {
            if token_account.amount != 1 {
                continue;
            }

            let position_addr = derive_position(&token_account.mint).0;
            let Ok(position) = self.client.get_anchor_account::<Position>(&position_addr).await else {
                continue;
            };

            if position.whirlpool != self.whirlpool_address {
                continue;
            }

            let mut ixns = vec![];

            if !Position::is_position_empty(&position) {
                ixns.push(self.ix.decrease_liquidity(
                    position.position_mint,
                    position.tick_lower_index,
                    position.tick_upper_index,
                    self.whirlpool.token_mint_a,
                    self.whirlpool.token_mint_b,
                    position.liquidity,
                    0,
                    0,
                ));
            }

            ixns.push(self.ix.close_position(position.position_mint));
            self.send_transaction(&ixns).await.unwrap();

            log::trace!("closed position for {position_addr}");
        }

        Ok(())
    }

    async fn set_wallet_liquidity(&self, token: &Pubkey, amount: u64) -> Result<(), ClientError> {
        let wallet_address = self.wallet.pubkey();
        let ata = get_associated_token_address(&wallet_address, token);

        let mut ixns = vec![];

        let request_amount = match self
            .client
            .try_get_packed_account::<TokenAccount>(&ata)
            .await
            .unwrap()
        {
            Some(account) => amount - account.amount,
            None => {
                ixns.push(create_associated_token_account(
                    &wallet_address,
                    &wallet_address,
                    token,
                    &spl_token::ID,
                ));

                amount
            }
        };

        if request_amount > 0 {
            ixns.push(token_request(&wallet_address, token, &ata, request_amount));
            self.send_transaction(&ixns).await.unwrap();
        }

        Ok(())
    }

    async fn send_transaction(&self, instructions: &[Instruction]) -> Result<(), ClientError> {
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.wallet.pubkey()),
            &[self.wallet],
            self.client.get_latest_blockhash().await.unwrap(),
        );

        self.client
            .send_and_confirm_transaction_legacy(&tx)
            .await
            .unwrap();

        Ok(())
    }

    async fn token_a_in_wallet(&self) -> Result<u64, ClientError> {
        let wallet_address = self.wallet.pubkey();
        let ata = get_associated_token_address(&wallet_address, &self.whirlpool.token_mint_a);
        let account = self.client.get_token_account(&ata).await.unwrap();

        Ok(account.amount)
    }

    async fn token_b_in_wallet(&self) -> Result<u64, ClientError> {
        let wallet_address = self.wallet.pubkey();
        let ata = get_associated_token_address(&wallet_address, &self.whirlpool.token_mint_b);
        let account = self.client.get_token_account(&ata).await.unwrap();

        Ok(account.amount)
    }

    async fn with_tick_array(
        &self,
        ixns: &mut Vec<Instruction>,
        tick_index: i32,
    ) -> Result<(), ClientError> {
        let tick_array = derive_tick_array(
            &self.whirlpool_address,
            tick_index,
            self.whirlpool.tick_spacing,
        );

        if !self.client.account_exists(&tick_array).await.unwrap() {
            log::trace!("need to create tick array for {tick_index}: {tick_array}");
            ixns.push(self.ix.initialize_tick_array(tick_index));
        }

        Ok(())
    }
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

fn price_to_sqrt_price(price: f64, decimals_a: impl Into<i64>, decimals_b: impl Into<i64>) -> u128 {
    let c = Decimal::TEN.pow(decimals_b.into() - decimals_a.into());
    let price = Decimal::from_f64_retain(price).unwrap();
    let sqrt_price = (price * c).sqrt().unwrap() * Decimal::from(2).powi(64);

    sqrt_price.floor().to_u128().unwrap()
}

fn sqrt_price_to_tick_index(sqrt_price: &u128, tick_spacing: u16) -> i32 {
    let tick_index = tick_index_from_sqrt_price(sqrt_price);
    tick_index - (tick_index % tick_spacing as i32)
}

fn max_liquidity_amount(decimals: u8, factor: u8) -> u64 {
    let n = decimals + factor;
    10u64.pow(n as u32)
}

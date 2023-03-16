use std::num::NonZeroU64;
use std::sync::Arc;

use anchor_lang::prelude::Rent;
use anchor_lang::Id;
use anchor_spl::dex::serum_dex::instruction::SelfTradeBehavior;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side};
use anchor_spl::dex::serum_dex::state::OpenOrders;
use anyhow::Error;

use anchor_spl::dex::{serum_dex, Dex};
use async_trait::async_trait;
use jet_margin_sdk::swap::openbook_swap::OpenBookMarket;
use jet_simulation::send_and_confirm;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::{system_instruction, sysvar::SysvarId};

use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::runtime::SolanaTestContext;
use crate::tokens::TokenManager;

#[async_trait]
pub trait OpenBookMarketConfig: Sized {
    async fn configure(
        ctx: &SolanaTestContext,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        base_lot_size: u64,
        quote_lot_size: u64,
        quote_dust_threshold: u64,
    ) -> Result<Self, Error>;

    async fn match_orders(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        limit: u16,
    ) -> Result<(), Error>;

    async fn consume_events(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        open_order_accounts: Vec<&Pubkey>,
        limit: u16,
    ) -> Result<(), Error>;

    async fn new_order(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        authority: &Keypair,
        open_orders: &Pubkey,
        order_payer: &Pubkey,
        params: OpenBookOrderParams,
    ) -> Result<(), Error>;

    async fn init_open_orders(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        open_orders: Keypair,
        authority: &Keypair,
    ) -> Result<Pubkey, Error>;
}

#[async_trait]
impl OpenBookMarketConfig for OpenBookMarket {
    /// Create a new OpenBook market
    async fn configure(
        ctx: &SolanaTestContext,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        base_lot_size: u64,
        quote_lot_size: u64,
        quote_dust_threshold: u64,
    ) -> anyhow::Result<Self> {
        // Initialize a market
        let token_manager = TokenManager::new(ctx.clone());

        let market = ctx.keygen.generate_key();
        let market_size = std::mem::size_of::<serum_dex::state::MarketState>() + 12;
        let market_lamports = ctx
            .rpc
            .get_minimum_balance_for_rent_exemption(market_size)
            .await?;
        let market_ix = system_instruction::create_account(
            &ctx.rpc.payer().pubkey(),
            &market.pubkey(),
            market_lamports,
            market_size as u64,
            &Dex::id(),
        );

        let (vault_nonce, vault_signer) = {
            let mut i = 0;
            loop {
                assert!(i < 100);
                if let Ok(pk) =
                    serum_dex::state::gen_vault_signer_key(i, &market.pubkey(), &Dex::id())
                {
                    break (i, pk);
                }
                i += 1;
            }
        };

        // State accounts
        let bid_ask_size = 65536 + 12;
        let bid_ask_lamports = ctx
            .rpc
            .get_minimum_balance_for_rent_exemption(bid_ask_size)
            .await?;
        let bids = ctx.keygen.generate_key();
        let asks = ctx.keygen.generate_key();
        let bids_ix = system_instruction::create_account(
            &ctx.rpc.payer().pubkey(),
            &bids.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            &Dex::id(),
        );
        let asks_ix = system_instruction::create_account(
            &ctx.rpc.payer().pubkey(),
            &asks.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            &Dex::id(),
        );

        let event_queue_size = 262144 + 12;
        let request_queue_size = 5120 + 12;
        let events_lamports = ctx
            .rpc
            .get_minimum_balance_for_rent_exemption(event_queue_size)
            .await?;
        let requests_lamports = ctx
            .rpc
            .get_minimum_balance_for_rent_exemption(request_queue_size)
            .await?;
        let events = ctx.keygen.generate_key();
        let requests = ctx.keygen.generate_key();
        let events_ix = system_instruction::create_account(
            &ctx.rpc.payer().pubkey(),
            &events.pubkey(),
            events_lamports,
            event_queue_size as u64,
            &Dex::id(),
        );
        let requests_ix = system_instruction::create_account(
            &ctx.rpc.payer().pubkey(),
            &requests.pubkey(),
            requests_lamports,
            request_queue_size as u64,
            &Dex::id(),
        );

        // Split transactions up
        send_and_confirm(
            &ctx.rpc,
            &[market_ix, bids_ix, asks_ix, events_ix, requests_ix],
            &[&market, &bids, &asks, &events, &requests],
        )
        .await?;

        let base_vault = token_manager
            .create_account(&base_mint, &vault_signer)
            .await?;
        let quote_vault = token_manager
            .create_account(&quote_mint, &vault_signer)
            .await?;

        let init_ix = serum_dex::instruction::initialize_market(
            &market.pubkey(),
            &Dex::id(),
            &base_mint,
            &quote_mint,
            &base_vault,
            &quote_vault,
            None,
            None,
            &bids.pubkey(),
            &asks.pubkey(),
            &requests.pubkey(),
            &events.pubkey(),
            base_lot_size,
            quote_lot_size,
            vault_nonce,
            quote_dust_threshold,
        )?;

        send_and_confirm(&ctx.rpc, &[init_ix], &[]).await?;

        let base_mint_decimals = token_manager.get_mint(&base_mint).await?.decimals;
        let quote_mint_decimals = token_manager.get_mint(&quote_mint).await?.decimals;

        Ok(Self {
            market: market.pubkey(),
            bids: bids.pubkey(),
            asks: asks.pubkey(),
            request_queue: requests.pubkey(),
            event_queue: events.pubkey(),
            base_mint,
            quote_mint,
            base_vault,
            quote_vault,
            vault_signer,
            program: Dex::id(),
            base_lot_size,
            quote_lot_size,
            base_mint_decimals,
            quote_mint_decimals,
        })
    }

    async fn init_open_orders(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        open_orders: Keypair,
        authority: &Keypair,
    ) -> Result<Pubkey, Error> {
        let open_orders_size = std::mem::size_of::<OpenOrders>() + 12;

        let open_orders_lamports = rpc
            .get_minimum_balance_for_rent_exemption(open_orders_size)
            .await?;

        let open_orders_ix = system_instruction::create_account(
            &rpc.payer().pubkey(),
            &open_orders.pubkey(),
            open_orders_lamports,
            open_orders_size as u64,
            &self.program,
        );

        let instruction = anchor_spl::dex::serum_dex::instruction::init_open_orders(
            &self.program,
            &open_orders.pubkey(),
            &authority.pubkey(),
            &self.market,
            None,
        )?;

        send_and_confirm(
            rpc,
            &[open_orders_ix, instruction],
            &[authority, &open_orders],
        )
        .await?;

        Ok(open_orders.pubkey())
    }

    async fn new_order(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        authority: &Keypair,
        open_orders: &Pubkey,
        order_payer: &Pubkey,
        params: OpenBookOrderParams,
    ) -> anyhow::Result<()> {
        let instruction = anchor_spl::dex::serum_dex::instruction::new_order(
            &self.market,
            open_orders,
            &self.request_queue,
            &self.event_queue,
            &self.bids,
            &self.asks,
            order_payer,
            &authority.pubkey(),
            &self.base_vault,
            &self.quote_vault,
            &spl_token::ID,
            &Rent::id(),
            None,
            &self.program,
            params.side,
            params.limit_price,
            params.max_coin_qty,
            params.order_type,
            params.client_order_id,
            params.self_trade_behavior,
            params.limit,
            params.max_native_pc_qty_including_fees,
        )?;

        send_and_confirm(rpc, &[instruction], &[authority]).await?;

        Ok(())
    }

    async fn match_orders(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        limit: u16,
    ) -> Result<(), Error> {
        let instruction = serum_dex::instruction::match_orders(
            &Dex::id(),
            &self.market,
            &self.request_queue,
            &self.bids,
            &self.asks,
            &self.event_queue,
            &base_fee_receivable,
            &quote_fee_receivable,
            limit,
        )?;

        send_and_confirm(rpc, &[instruction], &[]).await?;

        Ok(())
    }

    async fn consume_events(
        &self,
        rpc: &Arc<dyn SolanaRpcClient>,
        base_fee_receivable: Pubkey,
        quote_fee_receivable: Pubkey,
        open_order_accounts: Vec<&Pubkey>,
        limit: u16,
    ) -> anyhow::Result<()> {
        let instruction = serum_dex::instruction::consume_events(
            &Dex::id(),
            open_order_accounts,
            &self.market,
            &self.event_queue,
            &base_fee_receivable,
            &quote_fee_receivable,
            limit,
        )?;

        send_and_confirm(rpc, &[instruction], &[]).await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenBookOrderParams {
    pub side: Side,
    pub limit_price: NonZeroU64,
    pub max_coin_qty: NonZeroU64,
    pub order_type: OrderType,
    pub client_order_id: u64,
    pub self_trade_behavior: SelfTradeBehavior,
    pub limit: u16,
    pub max_native_pc_qty_including_fees: NonZeroU64,
}

/// Convert a price from quote tokens to lot sizes.
///
/// A USDC price of 1 will have 1_000_000 tokens as it has 6 decimals.
pub fn price_number_to_lot(
    price: u64,
    base_lamports: u64,
    base_lot_size: u64,
    quote_lot_size: u64,
) -> u64 {
    price
        .saturating_mul(base_lot_size)
        .saturating_div(base_lamports.saturating_mul(quote_lot_size))
}

#[test]
fn test_price_number_to_lot() {
    let base_lamports = 1_000_000_000;
    let base_lot_size = 100_000_000;
    let quote_lot_size = 100;

    let price = price_number_to_lot(36_500_000, base_lamports, base_lot_size, quote_lot_size);

    assert_eq!(price, 36500);

    // BTC
    let base_lamports = 1_000_000;
    let base_lot_size = 100;
    let quote_lot_size = 10;

    let price = price_number_to_lot(21_200_000_000, base_lamports, base_lot_size, quote_lot_size);

    assert_eq!(price, 212_000);
}

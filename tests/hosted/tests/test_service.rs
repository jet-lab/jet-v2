use std::collections::HashSet;

use anchor_lang::Id;
use jet_instructions::test_service::derive_token_mint;
use jet_margin_sdk::swap::openbook_swap::OpenBookMarket;
use jet_solana_client::rpc::AccountFilter;
use jet_test_service::TokenCreateParams;
use openbook::state::OpenOrders;
use solana_sdk::signature::Signer;
use solana_sdk::system_instruction;

use hosted_tests::margin_test_context;
use hosted_tests::openbook::OpenBookMarketConfig;

#[tokio::test]
async fn openorder_market_make() -> anyhow::Result<()> {
    let dex_program = anchor_spl::dex::id();
    // Get the mocked runtime
    let ctx = margin_test_context!();

    let mint_tsol = derive_token_mint("TSOL");
    let mint_usdc = derive_token_mint("USDC");

    let payer = ctx.payer().pubkey();

    // Register mints, tokens, oracles
    let mint_base_ix = jet_instructions::test_service::token_create(
        &payer,
        &TokenCreateParams {
            symbol: "USDC".to_string(),
            name: "USDC".to_string(),
            decimals: 6,
            authority: payer,
            oracle_authority: payer,
            max_amount: 100_000_000_000,
            source_symbol: "USDC".to_string(),
            price_ratio: 1.0,
        },
    );
    let mint_quote_ix = jet_instructions::test_service::token_create(
        &payer,
        &TokenCreateParams {
            symbol: "TSOL".to_string(),
            name: "TSOL".to_string(),
            decimals: 9,
            authority: payer,
            oracle_authority: payer,
            max_amount: 10_000_000_000_000,
            source_symbol: "SOL".to_string(),
            price_ratio: 1.0,
        },
    );

    let tx = ctx
        .rpc()
        .create_transaction(&[], &[mint_base_ix, mint_quote_ix])
        .await?;

    ctx.rpc().send_and_confirm_transaction(&tx).await?;

    // Create large accounts that can't be created as PDAs
    let bid_ask_size = 65536 + 12;
    let bid_ask_lamports = ctx
        .rpc()
        .get_minimum_balance_for_rent_exemption(bid_ask_size)
        .await?;
    let bids = ctx.solana.keygen.generate_key();
    let asks = ctx.solana.keygen.generate_key();
    let bids_ix = system_instruction::create_account(
        &payer,
        &bids.pubkey(),
        bid_ask_lamports,
        bid_ask_size as u64,
        &dex_program,
    );
    let asks_ix = system_instruction::create_account(
        &payer,
        &asks.pubkey(),
        bid_ask_lamports,
        bid_ask_size as u64,
        &dex_program,
    );
    let event_queue_size = 262144 + 12;
    let request_queue_size = 5120 + 12;
    let events_lamports = ctx
        .rpc()
        .get_minimum_balance_for_rent_exemption(event_queue_size)
        .await?;
    let requests_lamports = ctx
        .rpc()
        .get_minimum_balance_for_rent_exemption(request_queue_size)
        .await?;
    let events = ctx.solana.keygen.generate_key();
    let requests = ctx.solana.keygen.generate_key();
    let events_ix = system_instruction::create_account(
        &payer,
        &events.pubkey(),
        events_lamports,
        event_queue_size as u64,
        &dex_program,
    );
    let requests_ix = system_instruction::create_account(
        &payer,
        &requests.pubkey(),
        requests_lamports,
        request_queue_size as u64,
        &dex_program,
    );

    // Create a TSOL/USDC market
    let market_create_ix = jet_instructions::test_service::openbook_market_create(
        &dex_program,
        &payer,
        &mint_tsol,
        &mint_usdc,
        &bids.pubkey(),
        &asks.pubkey(),
        &events.pubkey(),
        &requests.pubkey(),
        500,
    );

    let tx = ctx
        .rpc()
        .create_transaction(
            &[&events, &requests, &bids, &asks],
            &[events_ix, requests_ix, bids_ix, asks_ix, market_create_ix],
        )
        .await?;
    ctx.rpc().send_and_confirm_transaction(&tx).await?;

    let mut supported_mints = HashSet::new();
    supported_mints.insert(mint_tsol);
    supported_mints.insert(mint_usdc);

    // There should be 1 openbook market
    let markets =
        OpenBookMarket::get_markets(&ctx.rpc(), &supported_mints, anchor_spl::dex::Dex::id())
            .await
            .unwrap();
    assert_eq!(markets.len(), 1);

    let market = markets.values().next().unwrap();

    // Get tokens
    let token_usdc = ctx.tokens().create_account(&mint_usdc, &payer).await?;
    let token_tsol = ctx.tokens().create_account(&mint_tsol, &payer).await?;

    // Set prices
    let oracle_usdc_ix = jet_instructions::test_service::token_update_pyth_price(
        &payer,
        &mint_usdc,
        101_000_000,
        5_000_000,
        -8,
    );
    let oracle_tsol_ix = jet_instructions::test_service::token_update_pyth_price(
        &payer,
        &mint_tsol,
        2_000_000_000,
        10_000_000,
        -8,
    );

    let tx = ctx
        .rpc()
        .create_transaction(
            &[],
            &[
                // base_tokens_ix,
                // quote_tokens_ix,
                oracle_usdc_ix,
                oracle_tsol_ix,
            ],
        )
        .await?;
    ctx.rpc().send_and_confirm_transaction(&tx).await?;

    // Cancel orders
    let cancel_orders_ix = jet_instructions::test_service::openbook_market_cancel_orders(
        &dex_program,
        &mint_tsol,
        &mint_usdc,
        &token_tsol,
        &token_usdc,
        &payer,
        &bids.pubkey(),
        &asks.pubkey(),
        &events.pubkey(),
    );

    let tx = ctx
        .rpc()
        .create_transaction(&[], &[cancel_orders_ix])
        .await?;
    ctx.rpc().send_and_confirm_transaction(&tx).await?;

    // Place orders
    let market_make_ix = jet_instructions::test_service::openbook_market_make(
        &dex_program,
        &mint_tsol,
        &mint_usdc,
        &token_tsol,
        &token_usdc,
        &payer,
        &bids.pubkey(),
        &asks.pubkey(),
        &requests.pubkey(),
        &events.pubkey(),
    );

    let tx = ctx.rpc().create_transaction(&[], &[market_make_ix]).await?;
    ctx.rpc().send_and_confirm_transaction(&tx).await?;

    // Find all open orders to consume
    let open_orders_accounts = ctx
        .solana
        .rpc
        .get_program_accounts(
            &dex_program,
            vec![AccountFilter::DataSize(
                12 + std::mem::size_of::<OpenOrders>(),
            )],
        )
        .await?;
    assert_eq!(open_orders_accounts.len(), 1);

    // Check that there are orders in the open orders account
    let open_orders_account =
        bytemuck::from_bytes::<OpenOrders>(&open_orders_accounts[0].1.data[12..]);

    // There can be up to 16 orders, sometimes there's fewer, so check 8
    let client_orders = &{ open_orders_account.client_order_ids }[..4];
    // Each order ID should be > 0
    for client_order_id in client_orders {
        assert_ne!(*client_order_id, 0);
    }

    market
        .match_orders(&ctx.rpc(), token_usdc, token_tsol, u16::MAX)
        .await?;
    market
        .consume_events(
            &ctx.rpc(),
            token_usdc,
            token_tsol,
            vec![&open_orders_accounts[0].0],
            u16::MAX,
        )
        .await?;

    Ok(())
}

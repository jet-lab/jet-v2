// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_lang::{InstructionData, ToAccountMetas};
use jet_fixed_term::seeds;
use jet_program_common::programs::ORCA_WHIRLPOOL;
use solana_sdk::{
    instruction::Instruction,
    pubkey,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::{self, SysvarId},
};

use jet_test_service::{
    seeds::{
        OPENBOOK_MARKET, OPENBOOK_MARKET_INFO, OPENBOOK_OPEN_ORDERS, ORCA_WHIRLPOOL_CONFIG,
        SWAP_POOL_FEES, SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_STATE, SWAP_POOL_TOKENS,
        TOKEN_INFO, TOKEN_MINT, TOKEN_PYTH_PRICE, TOKEN_PYTH_PRODUCT,
    },
    OpenBookMarketCancelOrdersParams, OpenBookMarketCreateParams, OpenBookMarketMakeParams,
    SaberSwapPoolCreateParams,
};

pub use jet_test_service::{SplSwapPoolCreateParams, TokenCreateParams};

pub use jet_test_service::ID as TEST_SERVICE_PROGRAM;

/// Get instruction to create a token as described
pub fn token_create(payer: &Pubkey, params: &TokenCreateParams) -> Instruction {
    let mint = derive_token_mint(&params.name);

    let accounts = jet_test_service::accounts::TokenCreate {
        payer: *payer,
        mint,
        info: derive_token_info(&mint),
        pyth_product: derive_pyth_product(&mint),
        pyth_price: derive_pyth_price(&mint),
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: Rent::id(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::TokenCreate {
            params: params.clone(),
        }
        .data(),
    }
}

/// Get instruction to register a token as described
pub fn token_register(payer: &Pubkey, mint: Pubkey, params: &TokenCreateParams) -> Instruction {
    let accounts = jet_test_service::accounts::TokenRegister {
        payer: *payer,
        mint,
        info: derive_token_info(&mint),
        pyth_product: derive_pyth_product(&mint),
        pyth_price: derive_pyth_price(&mint),
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: Rent::id(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::TokenRegister {
            params: params.clone(),
        }
        .data(),
    }
}

/// Get instruction to initialize native token
pub fn token_init_native(payer: &Pubkey, oracle_authority: &Pubkey) -> Instruction {
    let mint = spl_token::native_mint::ID;

    let accounts = jet_test_service::accounts::TokenInitNative {
        payer: *payer,
        mint,
        info: derive_token_info(&mint),
        pyth_product: derive_pyth_product(&mint),
        pyth_price: derive_pyth_price(&mint),
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: Rent::id(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::TokenInitNative {
            oracle_authority: *oracle_authority,
        }
        .data(),
    }
}

/// Request a number of tokens be minted
pub fn token_request(
    requester: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = jet_test_service::accounts::TokenRequest {
        requester: *requester,
        mint: *mint,
        info: derive_token_info(mint),
        destination: *destination,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::TokenRequest { amount }.data(),
    }
}

/// Update the pyth price for a token
pub fn token_update_pyth_price(
    authority: &Pubkey,
    mint: &Pubkey,
    price: i64,
    conf: i64,
    expo: i32,
) -> Instruction {
    let accounts = jet_test_service::accounts::TokenUpdatePythPrice {
        oracle_authority: *authority,
        info: derive_token_info(mint),
        pyth_price: derive_pyth_price(mint),
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::TokenUpdatePythPrice { price, conf, expo }.data(),
    }
}

/// Create the config for whirlpools
pub fn orca_whirlpool_create_config(
    payer: &Pubkey,
    authority: &Pubkey,
    default_fee_rate: u16,
) -> Instruction {
    let config = derive_whirlpool_config();

    let accounts = jet_test_service::accounts::OrcaWhirlpoolCreateConfig {
        config,
        system_program: system_program::ID,
        payer: *payer,
        whirlpool_program: ORCA_WHIRLPOOL,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::OrcaWhirlpoolCreateConfig {
            authority: *authority,
            default_fee_rate,
        }
        .data(),
    }
}

/// Create a swap pool
pub fn spl_swap_pool_create(
    swap_program: &Pubkey,
    payer: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    liquidity_level: u8,
    price_threshold: u16,
) -> Instruction {
    let addrs = derive_spl_swap_pool(swap_program, token_a, token_b);
    let accounts = jet_test_service::accounts::SplSwapPoolCreate {
        payer: *payer,
        mint_a: *token_a,
        mint_b: *token_b,
        info_a: derive_token_info(token_a),
        info_b: derive_token_info(token_b),
        pool_info: addrs.info,
        pool_state: addrs.state,
        pool_authority: addrs.authority,
        pool_mint: addrs.mint,
        pool_token_a: addrs.token_a_account,
        pool_token_b: addrs.token_b_account,
        pool_fees: addrs.fees,
        swap_program: *swap_program,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::SplSwapPoolCreate {
            params: SplSwapPoolCreateParams {
                liquidity_level,
                price_threshold,
                nonce: addrs.nonce,
            },
        }
        .data(),
    }
}

/// Balance an SPL swap pool
pub fn spl_swap_pool_balance(
    swap_program: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    scratch_a: &Pubkey,
    scratch_b: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let pool = derive_spl_swap_pool(swap_program, token_a, token_b);

    let accounts = jet_test_service::accounts::SplSwapPoolBalance {
        payer: *payer,
        scratch_a: *scratch_a,
        scratch_b: *scratch_b,
        mint_a: *token_a,
        mint_b: *token_b,
        info_a: derive_token_info(token_a),
        info_b: derive_token_info(token_b),
        pyth_price_a: derive_pyth_price(token_a),
        pyth_price_b: derive_pyth_price(token_b),
        pool_info: pool.info,
        pool_state: pool.state,
        pool_authority: pool.authority,
        pool_mint: pool.mint,
        pool_token_a: pool.token_a_account,
        pool_token_b: pool.token_b_account,
        pool_fees: pool.fees,
        swap_program: *swap_program,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::SplSwapPoolBalance {}.data(),
    }
}

/// Create a Saber swap pool
pub fn saber_swap_pool_create(
    swap_program: &Pubkey,
    payer: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    liquidity_level: u8,
    price_threshold: u16,
) -> Instruction {
    let addrs = derive_saber_swap_pool(swap_program, token_a, token_b);
    let accounts = jet_test_service::accounts::SaberSwapPoolCreate {
        payer: *payer,
        mint_a: *token_a,
        mint_b: *token_b,
        info_a: derive_token_info(token_a), // TODO: will clash
        info_b: derive_token_info(token_b),
        pool_info: addrs.info,
        pool_state: addrs.state,
        pool_authority: addrs.authority,
        pool_mint: addrs.mint,
        pool_token_a: addrs.token_a_account,
        pool_token_b: addrs.token_b_account,
        pool_fee_a: addrs.fee_a,
        pool_fee_b: addrs.fee_b,
        lp_token: addrs.lp_token,
        swap_program: *swap_program,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::SaberSwapPoolCreate {
            params: SaberSwapPoolCreateParams {
                nonce: addrs.nonce,
                liquidity_level,
                price_threshold,
            },
        }
        .data(),
    }
}

/// Balance a Saber swap pool
pub fn saber_swap_pool_balance(
    swap_program: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    scratch_a: &Pubkey,
    scratch_b: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let pool = derive_saber_swap_pool(swap_program, token_a, token_b);

    let accounts = jet_test_service::accounts::SaberSwapPoolBalance {
        payer: *payer,
        scratch_a: *scratch_a,
        scratch_b: *scratch_b,
        mint_a: *token_a,
        mint_b: *token_b,
        info_a: derive_token_info(token_a),
        info_b: derive_token_info(token_b),
        pyth_price_a: derive_pyth_price(token_a),
        pyth_price_b: derive_pyth_price(token_b),
        pool_info: pool.info,
        pool_state: pool.state,
        pool_authority: pool.authority,
        pool_mint: pool.mint,
        pool_token_a: pool.token_a_account,
        pool_token_b: pool.token_b_account,
        pool_fee_a: pool.fee_a,
        pool_fee_b: pool.fee_b,
        lp_token: pool.lp_token,
        saber_program: *swap_program,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::SaberSwapPoolBalance {}.data(),
    }
}

/// Create an Openbook market
#[allow(clippy::too_many_arguments)]
pub fn openbook_market_create(
    dex_program: &Pubkey,
    payer: &Pubkey,
    token_base: &Pubkey,
    token_quote: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    event_queue: &Pubkey,
    request_queue: &Pubkey,
    liquidity_amount: u64,
) -> Instruction {
    let addrs = derive_openbook_market(dex_program, token_base, token_quote, payer);
    let accounts = jet_test_service::accounts::OpenBookMarketCreate {
        payer: *payer,
        mint_base: *token_base,
        mint_quote: *token_quote,
        info_base: derive_token_info(token_base), // TODO: will clash
        info_quote: derive_token_info(token_quote),
        market_info: addrs.info,
        market_state: addrs.state,
        vault_signer: addrs.vault_signer,
        vault_base: addrs.vault_base,
        vault_quote: addrs.vault_quote,
        bids: *bids,
        asks: *asks,
        event_queue: *event_queue,
        request_queue: *request_queue,
        open_orders: addrs.open_orders,
        dex_program: *dex_program,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::OpenbookMarketCreate {
            params: OpenBookMarketCreateParams {
                vault_signer_nonce: addrs.vault_signer_nonce,
                base_lot_size: 1000, // This is a safe number for most markets
                quote_lot_size: 1,
                quote_dust_threshold: 1,
                liquidity_amount,
                initial_spread: 100,     // 1%
                incremental_spread: 200, // 2%
                basket_sizes: [1, 2, 3, 4, 5, 2, 2, 1],
            },
        }
        .data(),
    }
}

/// Cancel existing Openbook orders
#[allow(clippy::too_many_arguments)]
pub fn openbook_market_cancel_orders(
    dex_program: &Pubkey,
    token_base: &Pubkey,
    token_quote: &Pubkey,
    scratch_base: &Pubkey,
    scratch_quote: &Pubkey,
    payer: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    event_queue: &Pubkey,
) -> Instruction {
    let addrs = derive_openbook_market(dex_program, token_base, token_quote, payer);

    let accounts = jet_test_service::accounts::OpenBookMarketCancelOrders {
        payer: *payer,
        open_orders_owner: *payer,
        mint_base: *token_base,
        mint_quote: *token_quote,
        vault_base: addrs.vault_base,
        vault_quote: addrs.vault_quote,
        wallet_base: *scratch_base,
        wallet_quote: *scratch_quote,
        market_state: addrs.state,
        bids: *bids,
        asks: *asks,
        event_queue: *event_queue,
        open_orders: addrs.open_orders,
        dex_program: *dex_program,
        token_program: spl_token::ID,
        rent: sysvar::rent::ID,
        info_base: derive_token_info(token_base),
        info_quote: derive_token_info(token_quote),
        vault_signer: addrs.vault_signer,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::OpenbookMarketCancelOrders {
            params: OpenBookMarketCancelOrdersParams {
                bid_from_order_id: 100,
                ask_from_order_id: 200,
            },
        }
        .data(),
    }
}

/// Create Openbook orders
#[allow(clippy::too_many_arguments)]
pub fn openbook_market_make(
    dex_program: &Pubkey,
    token_base: &Pubkey,
    token_quote: &Pubkey,
    scratch_base: &Pubkey,
    scratch_quote: &Pubkey,
    payer: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    request_queue: &Pubkey,
    event_queue: &Pubkey,
) -> Instruction {
    let addrs = derive_openbook_market(dex_program, token_base, token_quote, payer);

    let accounts = jet_test_service::accounts::OpenBookMarketMake {
        payer: *payer,
        open_orders_owner: *payer,
        mint_base: *token_base,
        mint_quote: *token_quote,
        vault_base: addrs.vault_base,
        vault_quote: addrs.vault_quote,
        wallet_base: *scratch_base,
        wallet_quote: *scratch_quote,
        market_info: addrs.info,
        market_state: addrs.state,
        bids: *bids,
        asks: *asks,
        request_queue: *request_queue,
        event_queue: *event_queue,
        open_orders: addrs.open_orders,
        pyth_price_base: derive_pyth_price(token_base),
        pyth_price_quote: derive_pyth_price(token_quote),
        dex_program: *dex_program,
        token_program: spl_token::ID,
        rent: sysvar::rent::ID,
        info_base: derive_token_info(token_base),
        info_quote: derive_token_info(token_quote),
        vault_signer: addrs.vault_signer,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_test_service::ID,
        accounts,
        data: jet_test_service::instruction::OpenbookMarketMake {
            params: OpenBookMarketMakeParams {
                bid_from_order_id: 100,
                ask_from_order_id: 200,
            },
        }
        .data(),
    }
}

/// if the account is not initialized, invoke the instruction
pub fn if_not_initialized(account_to_check: Pubkey, ix: Instruction) -> Instruction {
    let mut accounts = jet_test_service::accounts::IfNotInitialized {
        program: ix.program_id,
        account_to_check,
    }
    .to_account_metas(None);

    accounts.extend(ix.accounts);

    Instruction {
        accounts,
        program_id: jet_test_service::ID,
        data: jet_test_service::instruction::IfNotInitialized {
            instruction: ix.data,
        }
        .data(),
    }
}

/// Get the token mint address for a given token name
pub fn derive_token_mint(name: &str) -> Pubkey {
    if name == "SOL" {
        return pubkey!("So11111111111111111111111111111111111111112");
    }

    Pubkey::find_program_address(&[TOKEN_MINT, name.as_bytes()], &jet_test_service::ID).0
}

/// Get the token info account
pub fn derive_token_info(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[TOKEN_INFO, mint.as_ref()], &jet_test_service::ID).0
}

/// Get the pyth product account
pub fn derive_pyth_product(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[TOKEN_PYTH_PRODUCT, mint.as_ref()], &jet_test_service::ID).0
}

/// Get the pyth price account
pub fn derive_pyth_price(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[TOKEN_PYTH_PRICE, mint.as_ref()], &jet_test_service::ID).0
}

/// Get the pyth price account
pub fn derive_ticket_mint(market: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[seeds::TICKET_MINT, market.as_ref()], &jet_fixed_term::ID).0
}

/// Get the Openbook open orders account
pub fn derive_openbook_open_orders(market: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[OPENBOOK_OPEN_ORDERS, market.as_ref(), owner.as_ref()],
        &jet_test_service::ID,
    )
    .0
}

/// Get the whirlpool config account
pub fn derive_whirlpool_config() -> Pubkey {
    Pubkey::find_program_address(&[ORCA_WHIRLPOOL_CONFIG], &jet_test_service::ID).0
}

/// Get the addresses for a swap pool
pub fn derive_spl_swap_pool(
    program: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
) -> SplSwapPoolAddress {
    let info = Pubkey::find_program_address(
        &[SWAP_POOL_INFO, token_a.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let state = Pubkey::find_program_address(
        &[SWAP_POOL_STATE, token_a.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let (authority, nonce) = Pubkey::find_program_address(&[state.as_ref()], program);
    let token_a_account = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_a.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let token_b_account = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let mint =
        Pubkey::find_program_address(&[SWAP_POOL_MINT, state.as_ref()], &jet_test_service::ID).0;
    let fees = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), mint.as_ref()],
        &jet_test_service::ID,
    )
    .0;

    SplSwapPoolAddress {
        info,
        state,
        authority,
        token_a_account,
        token_b_account,
        mint,
        fees,
        nonce,
    }
}

/// Get the addresses for a Saber swap pool
pub fn derive_saber_swap_pool(
    program: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
) -> SaberSwapPoolAddress {
    let info = Pubkey::find_program_address(
        &[SWAP_POOL_INFO, token_a.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let state = Pubkey::find_program_address(
        &[SWAP_POOL_STATE, token_a.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let (authority, nonce) = Pubkey::find_program_address(&[state.as_ref()], program);
    let token_a_account = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_a.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let token_b_account = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let mint =
        Pubkey::find_program_address(&[SWAP_POOL_MINT, state.as_ref()], &jet_test_service::ID).0;
    let fee_a = Pubkey::find_program_address(
        &[SWAP_POOL_FEES, state.as_ref(), token_a.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let fee_b = Pubkey::find_program_address(
        &[SWAP_POOL_FEES, state.as_ref(), token_b.as_ref()],
        &jet_test_service::ID,
    )
    .0;

    let lp_destination = Pubkey::find_program_address(
        &[SWAP_POOL_FEES, state.as_ref(), mint.as_ref()],
        &jet_test_service::ID,
    )
    .0;

    SaberSwapPoolAddress {
        info,
        state,
        authority,
        token_a_account,
        token_b_account,
        mint,
        fee_a,
        fee_b,
        lp_token: lp_destination,
        nonce,
    }
}

/// Get the addresses for a Saber swap pool
pub fn derive_openbook_market(
    program: &Pubkey,
    token_base: &Pubkey,
    token_quote: &Pubkey,
    payer: &Pubkey,
) -> OpenbookMarketAddresses {
    let info = Pubkey::find_program_address(
        &[
            OPENBOOK_MARKET_INFO,
            token_base.as_ref(),
            token_quote.as_ref(),
        ],
        &jet_test_service::ID,
    )
    .0;
    let state = Pubkey::find_program_address(
        &[OPENBOOK_MARKET, token_base.as_ref(), token_quote.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let (vault_nonce, vault_signer) = {
        let mut i = 0;
        loop {
            assert!(i < 100);
            if let Ok(pk) =
                anchor_spl::dex::serum_dex::state::gen_vault_signer_key(i, &state, program)
            {
                break (i, pk);
            }
            i += 1;
        }
    };
    let vault_base = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_base.as_ref()],
        &jet_test_service::ID,
    )
    .0;
    let vault_quote = Pubkey::find_program_address(
        &[SWAP_POOL_TOKENS, state.as_ref(), token_quote.as_ref()],
        &jet_test_service::ID,
    )
    .0;

    let open_orders = derive_openbook_open_orders(&state, payer);

    OpenbookMarketAddresses {
        info,
        state,
        vault_base,
        vault_quote,
        vault_signer,
        open_orders,
        vault_signer_nonce: vault_nonce,
    }
}

/// Set of addresses for a test swap pool
pub struct SplSwapPoolAddress {
    /// The test-service state about the pool
    pub info: Pubkey,

    /// The address of the swap pool state
    pub state: Pubkey,

    /// The authority
    pub authority: Pubkey,

    /// The token A vault
    pub token_a_account: Pubkey,

    /// The token B vault
    pub token_b_account: Pubkey,

    /// The LP token
    pub mint: Pubkey,

    /// The account to collect fees
    pub fees: Pubkey,

    /// The pool nonce
    pub nonce: u8,
}

/// Set of addresses for a test Saber swap pool
pub struct SaberSwapPoolAddress {
    /// The test-service state about the pool
    pub info: Pubkey,

    /// The address of the swap pool state
    pub state: Pubkey,

    /// The authority
    pub authority: Pubkey,

    /// The token A vault
    pub token_a_account: Pubkey,

    /// The token B vault
    pub token_b_account: Pubkey,

    /// The LP token
    pub mint: Pubkey,

    /// The account to collect fees from token A
    pub fee_a: Pubkey,

    /// The account to collect fees from token B
    pub fee_b: Pubkey,

    /// The account to transfer liquiditity token to/from
    pub lp_token: Pubkey,

    /// The pool nonce
    pub nonce: u8,
}

/// Set of addressess for a test openbook market
pub struct OpenbookMarketAddresses {
    /// The test-service state about the pool
    pub info: Pubkey,

    /// The address of the swap pool state
    pub state: Pubkey,

    /// The token A vault
    pub vault_base: Pubkey,

    /// The token B vault
    pub vault_quote: Pubkey,

    /// The vault signer
    pub vault_signer: Pubkey,

    /// Open orders
    pub open_orders: Pubkey,

    /// The vault signer nonce
    pub vault_signer_nonce: u64,
}

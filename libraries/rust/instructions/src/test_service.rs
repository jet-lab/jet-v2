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
        SWAP_POOL_INFO, SWAP_POOL_MINT, SWAP_POOL_STATE, SWAP_POOL_TOKENS, TOKEN_INFO, TOKEN_MINT,
        TOKEN_PYTH_PRICE, TOKEN_PYTH_PRODUCT,
    },
    SplSwapPoolCreateParams, TokenCreateParams,
};

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

/// Create a swap pool
pub fn spl_swap_pool_create(
    payer: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    liquidity_level: u8,
    price_threshold: u16,
) -> Instruction {
    let addrs = derive_swap_pool(token_a, token_b);
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
        swap_program: spl_token_swap::ID,
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
    token_a: &Pubkey,
    token_b: &Pubkey,
    scratch_a: &Pubkey,
    scratch_b: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let pool = derive_swap_pool(token_a, token_b);

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
        swap_program: spl_token_swap::ID,
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

// todo - fixme: orca whirlpool initialization
/// Create an Orca whirlpool
pub fn orca_whirlpool_create(
    payer: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    liquidity_level: u8,
    price_threshold: u16,
) -> Instruction {
    let addrs = derive_swap_pool(token_a, token_b);
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
        swap_program: spl_token_swap::ID,
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

/// Balance an Orca whirlpool
pub fn orca_whirlpool_balance(
    token_a: &Pubkey,
    token_b: &Pubkey,
    scratch_a: &Pubkey,
    scratch_b: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let pool = derive_swap_pool(token_a, token_b);

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
        swap_program: spl_token_swap::ID,
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

/// Get the addresses for a swap pool
pub fn derive_swap_pool(token_a: &Pubkey, token_b: &Pubkey) -> SwapPoolAddress {
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
    let (authority, nonce) = Pubkey::find_program_address(&[state.as_ref()], &spl_token_swap::ID);
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

    SwapPoolAddress {
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

/// Set of addresses for a test swap pool
pub struct SwapPoolAddress {
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

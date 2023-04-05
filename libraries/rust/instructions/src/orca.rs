#![allow(clippy::too_many_arguments)]
use anchor_lang::{InstructionData, ToAccountMetas};
use orca_whirlpool::state::{
    OpenPositionBumps, Whirlpool, WhirlpoolBumps, MAX_TICK_INDEX, MIN_TICK_INDEX, TICK_ARRAY_SIZE,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar};
use spl_associated_token_account::get_associated_token_address;

pub use orca_whirlpool::ID as ORCA_WHIRLPOOL_PROGRAM;

/// A builder for Orca Whirlpools
pub struct WhirlpoolIxBuilder {
    pub config: Pubkey,
    pub base: Pubkey,
    pub quote: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub whirlpool: Pubkey,

    tick_spacing: u16,
    payer: Pubkey,
    bump: u8,
}

impl WhirlpoolIxBuilder {
    pub fn new(
        payer: Pubkey,
        config: Pubkey,
        base: Pubkey,
        quote: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        tick_spacing: u16,
    ) -> Self {
        let (whirlpool, bump) = derive_whirlpool(&config, &base, &quote, tick_spacing);

        Self {
            payer,
            config,
            base,
            quote,
            base_vault,
            quote_vault,
            tick_spacing,
            whirlpool,
            bump,
        }
    }

    pub fn from_whirlpool(payer: Pubkey, address: Pubkey, whirlpool: &Whirlpool) -> Self {
        Self {
            payer,
            config: whirlpool.whirlpools_config,
            base: whirlpool.token_mint_a,
            quote: whirlpool.token_mint_b,
            base_vault: whirlpool.token_vault_a,
            quote_vault: whirlpool.token_vault_b,
            tick_spacing: whirlpool.tick_spacing,
            whirlpool: address,
            bump: whirlpool.whirlpool_bump[0],
        }
    }

    pub fn initialize_pool(&self, initial_sqrt_price: u128) -> Instruction {
        let accounts = orca_whirlpool::accounts::InitializePool {
            whirlpools_config: self.config,
            funder: self.payer,
            token_mint_a: self.base,
            token_mint_b: self.quote,
            whirlpool: self.whirlpool,
            token_vault_a: self.base_vault,
            token_vault_b: self.quote_vault,
            fee_tier: derive_fee_tier(&self.config, self.tick_spacing),
            token_program: spl_token::ID,
            rent: sysvar::rent::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::InitializePool {
                bumps: WhirlpoolBumps {
                    whirlpool_bump: self.bump,
                },
                tick_spacing: self.tick_spacing,
                initial_sqrt_price,
            }
            .data(),
            accounts,
        }
    }

    pub fn initialize_tick_array(&self, tick_index: i32) -> Instruction {
        let start_tick_index = start_tick_index(tick_index, self.tick_spacing, 0);
        let accounts = orca_whirlpool::accounts::InitializeTickArray {
            funder: self.payer,
            whirlpool: self.whirlpool,
            tick_array: derive_tick_array(&self.whirlpool, start_tick_index, self.tick_spacing),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::InitializeTickArray { start_tick_index }.data(),
            accounts,
        }
    }

    pub fn open_position(
        &self,
        mint: Pubkey,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Instruction {
        assert_eq!(tick_lower_index % self.tick_spacing as i32, 0);
        assert_eq!(tick_upper_index % self.tick_spacing as i32, 0);

        let (position, position_bump) = derive_position(&mint);
        let position_token_account = get_associated_token_address(&self.payer, &mint);

        let accounts = orca_whirlpool::accounts::OpenPosition {
            funder: self.payer,
            whirlpool: self.whirlpool,
            system_program: system_program::ID,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            rent: sysvar::rent::ID,
            owner: self.payer,
            position_mint: mint,
            position_token_account,
            position,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::OpenPosition {
                bumps: OpenPositionBumps { position_bump },
                tick_lower_index,
                tick_upper_index,
            }
            .data(),
            accounts,
        }
    }

    pub fn close_position(&self, mint: Pubkey) -> Instruction {
        let (position, _) = derive_position(&mint);
        let position_token_account = get_associated_token_address(&self.payer, &mint);

        let accounts = orca_whirlpool::accounts::ClosePosition {
            receiver: self.payer,
            token_program: spl_token::ID,
            position_mint: mint,
            position_token_account,
            position_authority: self.payer,
            position,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::ClosePosition {}.data(),
            accounts,
        }
    }

    pub fn increase_liquidity(
        &self,
        mint: Pubkey,
        tick_lower_index: i32,
        tick_upper_index: i32,
        base_source_account: Pubkey,
        quote_source_account: Pubkey,
        liquidity_amount: u128,
        base_max_amount: u64,
        quote_max_amount: u64,
    ) -> Instruction {
        let (position, _) = derive_position(&mint);
        let position_token_account = get_associated_token_address(&self.payer, &mint);

        let accounts = orca_whirlpool::accounts::ModifyLiquidity {
            whirlpool: self.whirlpool,
            token_program: spl_token::ID,
            position_authority: self.payer,
            tick_array_lower: derive_tick_array(
                &self.whirlpool,
                tick_lower_index,
                self.tick_spacing,
            ),
            tick_array_upper: derive_tick_array(
                &self.whirlpool,
                tick_upper_index,
                self.tick_spacing,
            ),
            token_vault_a: self.base_vault,
            token_vault_b: self.quote_vault,
            token_owner_account_a: base_source_account,
            token_owner_account_b: quote_source_account,
            position_token_account,
            position,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::IncreaseLiquidity {
                liquidity_amount,
                token_max_a: base_max_amount,
                token_max_b: quote_max_amount,
            }
            .data(),
            accounts,
        }
    }

    pub fn decrease_liquidity(
        &self,
        mint: Pubkey,
        tick_lower_index: i32,
        tick_upper_index: i32,
        base_target_account: Pubkey,
        quote_target_account: Pubkey,
        liquidity_amount: u128,
        base_min_amount: u64,
        quote_min_amount: u64,
    ) -> Instruction {
        let (position, _) = derive_position(&mint);
        let position_token_account = get_associated_token_address(&self.payer, &mint);

        let accounts = orca_whirlpool::accounts::ModifyLiquidity {
            whirlpool: self.whirlpool,
            token_program: spl_token::ID,
            position_authority: self.payer,
            tick_array_lower: derive_tick_array(
                &self.whirlpool,
                tick_lower_index,
                self.tick_spacing,
            ),
            tick_array_upper: derive_tick_array(
                &self.whirlpool,
                tick_upper_index,
                self.tick_spacing,
            ),
            token_vault_a: self.base_vault,
            token_vault_b: self.quote_vault,
            token_owner_account_a: base_target_account,
            token_owner_account_b: quote_target_account,
            position_token_account,
            position,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::DecreaseLiquidity {
                liquidity_amount,
                token_min_a: base_min_amount,
                token_min_b: quote_min_amount,
            }
            .data(),
            accounts,
        }
    }

    pub fn swap(
        &self,
        base_account: Pubkey,
        quote_account: Pubkey,
        ticks: [i32; 3],
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Instruction {
        let accounts = orca_whirlpool::accounts::Swap {
            whirlpool: self.whirlpool,
            oracle: derive_whirlpool_oracle(&self.whirlpool),
            tick_array_0: derive_tick_array(&self.whirlpool, ticks[0], self.tick_spacing),
            tick_array_1: derive_tick_array(&self.whirlpool, ticks[1], self.tick_spacing),
            tick_array_2: derive_tick_array(&self.whirlpool, ticks[2], self.tick_spacing),
            token_authority: self.payer,
            token_owner_account_a: base_account,
            token_owner_account_b: quote_account,
            token_vault_a: self.base_vault,
            token_vault_b: self.quote_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            data: orca_whirlpool::instruction::Swap {
                amount,
                other_amount_threshold,
                sqrt_price_limit,
                amount_specified_is_input,
                a_to_b,
            }
            .data(),
            accounts,
        }
    }
}

pub fn whirlpool_initialize_fee_tier(
    payer: &Pubkey,
    authority: &Pubkey,
    config: &Pubkey,
    tick_spacing: u16,
    default_fee_rate: u16,
) -> Instruction {
    let accounts = orca_whirlpool::accounts::InitializeFeeTier {
        config: *config,
        funder: *payer,
        fee_tier: derive_fee_tier(config, tick_spacing),
        fee_authority: *authority,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: orca_whirlpool::ID,
        data: orca_whirlpool::instruction::InitializeFeeTier {
            default_fee_rate,
            tick_spacing,
        }
        .data(),
        accounts,
    }
}

pub fn derive_whirlpool(
    config: &Pubkey,
    base: &Pubkey,
    quote: &Pubkey,
    tick_spacing: u16,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"whirlpool",
            config.as_ref(),
            base.as_ref(),
            quote.as_ref(),
            tick_spacing.to_le_bytes().as_ref(),
        ],
        &orca_whirlpool::ID,
    )
}

pub fn derive_fee_tier(config: &Pubkey, tick_spacing: u16) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"fee_tier",
            config.as_ref(),
            tick_spacing.to_le_bytes().as_ref(),
        ],
        &orca_whirlpool::ID,
    )
    .0
}

pub fn derive_tick_array(whirlpool: &Pubkey, tick_index: i32, tick_spacing: u16) -> Pubkey {
    assert!(tick_index >= MIN_TICK_INDEX);
    assert!(tick_index <= MAX_TICK_INDEX);

    let start_tick_index = start_tick_index(tick_index, tick_spacing, 0);

    Pubkey::find_program_address(
        &[
            b"tick_array",
            whirlpool.as_ref(),
            start_tick_index.to_string().as_bytes(),
        ],
        &orca_whirlpool::ID,
    )
    .0
}

pub fn derive_position(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"position", mint.as_ref()], &orca_whirlpool::ID)
}

pub fn derive_whirlpool_oracle(whirlpool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"oracle", whirlpool.as_ref()], &orca_whirlpool::ID).0
}

pub fn start_tick_index(tick_index: i32, tick_spacing: u16, offset: i32) -> i32 {
    let index_real = tick_index as f64 / tick_spacing as f64 / TICK_ARRAY_SIZE as f64;
    (index_real.floor() as i32 + offset) * tick_spacing as i32 * TICK_ARRAY_SIZE
}

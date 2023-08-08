use std::{collections::HashSet, time::SystemTime};

use anchor_lang::{prelude::Id, system_program::System, InstructionData, ToAccountMetas};
use orca_whirlpool::{
    math::sqrt_price_from_tick_index,
    state::{OpenPositionBumps, Position as WhirlpoolPosition},
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::SysvarId,
};
use spl_associated_token_account::get_associated_token_address;

use jet_margin_orca::{accounts as ix_accounts, instruction as ix_data, seeds, WhirlpoolConfig};

use crate::{
    margin::{derive_adapter_config, derive_token_config},
    orca::derive_tick_array,
};

pub use jet_margin_orca::ID as MARGIN_ORCA_PROGRAM;

// Re-export the Whirlpool so dependents don't have to import the crate
pub use orca_whirlpool::state::Whirlpool;

/// Utility for creating instructions to interact with the Orca adapter for a
/// specific Whirlpool.
#[derive(Clone)]
pub struct MarginOrcaIxBuilder {
    pub airspace: Pubkey,
    pub address: Pubkey,
    pub margin_position_mint: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub oracle_a: Pubkey,
    pub oracle_b: Pubkey,
}

impl MarginOrcaIxBuilder {
    pub fn new(
        airspace: Pubkey,
        token_a: Pubkey,
        token_b: Pubkey,
        oracle_a: Pubkey,
        oracle_b: Pubkey,
    ) -> Self {
        let address = derive::derive_margin_orca_config(&airspace, &token_a, &token_b);
        let liquidity_mint = derive::derive_position_mint(&address);

        Self {
            airspace,
            address,
            margin_position_mint: liquidity_mint,
            token_a,
            token_b,
            oracle_a,
            oracle_b,
        }
    }

    pub fn new_from_config(config: &WhirlpoolConfig) -> Self {
        Self::new(
            config.airspace,
            config.mint_a,
            config.mint_b,
            config.token_a_oracle,
            config.token_b_oracle,
        )
    }

    pub fn create(&self, payer: Pubkey, authority: Pubkey) -> Instruction {
        let adapter_config = derive_adapter_config(&self.airspace, &MARGIN_ORCA_PROGRAM);
        let token_a_config = derive_token_config(&self.airspace, &self.token_a);
        let token_b_config = derive_token_config(&self.airspace, &self.token_b);
        let accounts = ix_accounts::CreateWhirlpoolConfig {
            payer,
            authority,
            airspace: self.airspace,
            adapter_config,
            whirlpool_config: self.address,
            token_a_config,
            token_b_config,
            margin_position_mint: self.margin_position_mint,
            orca_program: orca_whirlpool::ID,
            token_program: spl_token::ID,
            system_program: System::id(),
            mint_a: self.token_a,
            mint_b: self.token_b,
        }
        .to_account_metas(None);

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            data: ix_data::CreateWhirlpoolConfig {}.data(),
            accounts,
        }
    }

    // Register meta
    pub fn register_margin_position(&self, margin_account: Pubkey, payer: Pubkey) -> Instruction {
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);
        let margin_position = derive::derive_margin_position(&margin_account, &self.address);

        let accounts = ix_accounts::RegisterMarginPosition {
            payer,
            owner: margin_account,
            adapter_position_metadata,
            whirlpool_config: self.address,
            position_token_config: derive_token_config(&self.airspace, &self.margin_position_mint),
            margin_position,
            margin_position_mint: self.margin_position_mint,
            token_program: spl_token::ID,
            system_program: System::id(),
            rent: Rent::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            accounts,
            data: ix_data::RegisterMarginPosition {}.data(),
        }
    }

    // Close position meta
    pub fn close_position_meta(&self, margin_account: Pubkey, receiver: Pubkey) -> Instruction {
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);
        let margin_position = derive::derive_margin_position(&margin_account, &self.address);

        let accounts = ix_accounts::ClosePositionMeta {
            receiver,
            owner: margin_account,
            whirlpool_config: self.address,
            adapter_position_metadata,
            margin_position,
            margin_position_mint: self.margin_position_mint,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            accounts,
            data: ix_data::ClosePositionMeta {}.data(),
        }
    }

    // Create a whirlpool position returning the position mint and position account
    pub fn open_whirlpool_position(
        &self,
        margin_account: Pubkey,
        payer: Pubkey,
        whirlpool_address: Pubkey,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> (Instruction, Pubkey, Pubkey) {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let position_mint =
            derive::derive_whirlpool_mint(&margin_account, &whirlpool_address, seed);
        let (position, position_bump) = crate::orca::derive_position(&position_mint);
        let position_token_account = get_associated_token_address(&margin_account, &position_mint);
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);
        let margin_position = derive::derive_margin_position(&margin_account, &self.address);

        let accounts = ix_accounts::OpenWhirlpoolPosition {
            payer,
            owner: margin_account,
            adapter_position_metadata,
            position,
            position_mint,
            position_token_account,
            whirlpool: whirlpool_address,
            orca_program: orca_whirlpool::ID,
            token_program: spl_token::ID,
            system_program: System::id(),
            rent: Rent::id(),
            associated_token_program: spl_associated_token_account::ID,
            whirlpool_config: self.address,
            margin_position,
            margin_position_mint: self.margin_position_mint,
        }
        .to_account_metas(None);

        (
            Instruction {
                program_id: MARGIN_ORCA_PROGRAM,
                accounts,
                data: ix_data::OpenWhirlpoolPosition {
                    bumps: OpenPositionBumps { position_bump },
                    seed,
                    tick_lower_index,
                    tick_upper_index,
                }
                .data(),
            },
            position_mint,
            position,
        )
    }

    // Close position
    pub fn close_whirlpool_position(
        &self,
        margin_account: Pubkey,
        receiver: Pubkey,
        mint: Pubkey,
    ) -> Instruction {
        let (position, _) = crate::orca::derive_position(&mint);
        let position_token_account = get_associated_token_address(&margin_account, &mint);
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);
        let margin_position = derive::derive_margin_position(&margin_account, &self.address);

        let accounts = ix_accounts::CloseWhirlpoolPosition {
            receiver,
            owner: margin_account,
            adapter_position_metadata,
            position,
            position_mint: mint,
            position_token_account,
            orca_program: orca_whirlpool::ID,
            token_program: spl_token::ID,
            whirlpool_config: self.address,
            margin_position,
            margin_position_mint: self.margin_position_mint,
        }
        .to_account_metas(None);

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            accounts,
            data: ix_data::CloseWhirlpoolPosition {}.data(),
        }
    }
    // Add liquidity
    #[allow(clippy::too_many_arguments)]
    pub fn add_liquidity(
        &self,
        margin_account: Pubkey,
        pool_summary: &WhirlpoolSummary,
        position_summary: &WhirlpoolPositionSummary,
        whirlpools: &HashSet<Pubkey>,
        positions: &HashSet<Pubkey>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Instruction {
        self.modify_liquidity(
            margin_account,
            pool_summary,
            position_summary,
            whirlpools,
            positions,
            liquidity_amount,
            token_max_a,
            token_max_b,
            true,
        )
    }

    // Remove liquidity
    #[allow(clippy::too_many_arguments)]
    pub fn remove_liquidity(
        &self,
        margin_account: Pubkey,
        pool_summary: &WhirlpoolSummary,
        position_summary: &WhirlpoolPositionSummary,
        whirlpools: &HashSet<Pubkey>,
        positions: &HashSet<Pubkey>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Instruction {
        self.modify_liquidity(
            margin_account,
            pool_summary,
            position_summary,
            whirlpools,
            positions,
            liquidity_amount,
            token_max_a,
            token_max_b,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn modify_liquidity(
        &self,
        margin_account: Pubkey,
        pool_summary: &WhirlpoolSummary,
        position_summary: &WhirlpoolPositionSummary,
        whirlpools: &HashSet<Pubkey>,
        positions: &HashSet<Pubkey>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
        is_increase: bool,
    ) -> Instruction {
        let mint = position_summary.position_mint;
        let (position, _) = crate::orca::derive_position(&mint);
        let position_token_account = get_associated_token_address(&margin_account, &mint);
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);

        let token_owner_a = get_associated_token_address(&margin_account, &self.token_a);
        let token_owner_b = get_associated_token_address(&margin_account, &self.token_b);

        let mut accounts = ix_accounts::ModifyLiquidity {
            owner: margin_account,
            adapter_position_metadata,
            position,
            position_token_account,
            orca_program: orca_whirlpool::ID,
            token_program: spl_token::ID,
            whirlpool: pool_summary.address,
            whirlpool_config: self.address,
            token_owner_account_a: token_owner_a,
            token_owner_account_b: token_owner_b,
            token_vault_a: pool_summary.vault_a,
            token_vault_b: pool_summary.vault_b,
            token_a_oracle: self.oracle_a,
            token_b_oracle: self.oracle_b,
            tick_array_lower: position_summary.tick_array_lower,
            tick_array_upper: position_summary.tick_array_upper,
        }
        .to_account_metas(None);

        accounts.extend_from_slice(
            &whirlpools
                .iter()
                .map(|address| AccountMeta::new_readonly(*address, false))
                .collect::<Vec<_>>(),
        );

        accounts.extend_from_slice(
            &positions
                .iter()
                .map(|address| AccountMeta::new_readonly(*address, false))
                .collect::<Vec<_>>(),
        );

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            accounts,
            data: if is_increase {
                ix_data::IncreaseLiquidity {
                    liquidity_amount,
                    token_max_a,
                    token_max_b,
                }
                .data()
            } else {
                ix_data::DecreaseLiquidity {
                    liquidity_amount,
                    token_max_a,
                    token_max_b,
                }
                .data()
            },
        }
    }

    // Refresh position
    pub fn margin_refresh_position(
        &self,
        margin_account: Pubkey,
        whirlpools: &HashSet<Pubkey>,
        positions: &HashSet<Pubkey>,
    ) -> Instruction {
        let adapter_position_metadata =
            derive::derive_adapter_position_metadata(&margin_account, &self.address);

        let mut accounts = ix_accounts::MarginRefreshPosition {
            owner: margin_account,
            adapter_position_metadata,
            whirlpool_config: self.address,
            token_a_oracle: self.oracle_a,
            token_b_oracle: self.oracle_b,
        }
        .to_account_metas(None);

        accounts.extend_from_slice(
            &whirlpools
                .iter()
                .map(|address| AccountMeta::new_readonly(*address, false))
                .collect::<Vec<_>>(),
        );

        accounts.extend_from_slice(
            &positions
                .iter()
                .map(|address| AccountMeta::new_readonly(*address, false))
                .collect::<Vec<_>>(),
        );

        Instruction {
            program_id: MARGIN_ORCA_PROGRAM,
            accounts,
            data: ix_data::MarginRefreshPosition {}.data(),
        }
    }

    /// Refresh positions by updating entitled fees and rewards
    pub fn update_fees_and_rewards(&self, position: &WhirlpoolPositionSummary) -> Instruction {
        let accounts = orca_whirlpool::accounts::UpdateFeesAndRewards {
            whirlpool: position.whirlpool,
            position: position.position,
            tick_array_lower: position.tick_array_lower,
            tick_array_upper: position.tick_array_upper,
        }
        .to_account_metas(None);

        Instruction {
            program_id: orca_whirlpool::ID,
            accounts,
            data: orca_whirlpool::instruction::UpdateFeesAndRewards {}.data(),
        }
    }

    // Collect reward
    pub fn collect_reward(&self, margin_account: Pubkey) -> Instruction {
        unimplemented!("TODO {margin_account}");
        // let adapter_position_metadata =
        //     derive::derive_adapter_position_metadata(&margin_account, &self.address);

        // let accounts = ix_accounts::CollectReward {
        //     whirlpool: todo!(),
        //     position_authority: todo!(),
        //     position: todo!(),
        //     position_token_account: todo!(),
        //     reward_owner_account: margin_account,
        //     reward_vault: todo!(),
        //     orca_program: orca_whirlpool::ID,
        //     token_program: spl_token::ID,
        // }
        // .to_account_metas(None);

        // Instruction {
        //     program_id: MARGIN_ORCA_PROGRAM,
        //     accounts,
        //     data: ix_data::CollectReward {
        //         reward_index: todo!(),
        //     }
        //     .data(),
        // }
    }
}

/// The minimal information required when constructing Whirlpool instructions
#[derive(Clone)]
pub struct WhirlpoolSummary {
    pub address: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub tick_spacing: u16,
    /// The current tick index, should be updated regularly
    pub current_tick_index: i32,
    /// The current sqrt price, should be updated regularly
    pub current_sqrt_price: u128,
    // TODO: reward info
}

impl From<(Pubkey, &Whirlpool)> for WhirlpoolSummary {
    fn from(value: (Pubkey, &Whirlpool)) -> Self {
        let (address, pool) = value;
        Self {
            address,
            vault_a: pool.token_vault_a,
            vault_b: pool.token_vault_b,
            tick_spacing: pool.tick_spacing,
            current_tick_index: pool.tick_current_index,
            current_sqrt_price: pool.sqrt_price,
        }
    }
}

impl WhirlpoolSummary {
    pub fn derive_tick_array(&self, tick_index: i32) -> Pubkey {
        derive_tick_array(&self.address, tick_index, self.tick_spacing)
    }
}

#[derive(Clone)]
pub struct WhirlpoolPositionSummary {
    pub position: Pubkey,
    pub position_mint: Pubkey,
    pub whirlpool: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
}

impl WhirlpoolPositionSummary {
    pub fn new(
        position: Pubkey,
        position_mint: Pubkey,
        whirlpool: impl Into<WhirlpoolSummary>,
        tick_lower_index: i32,
        tick_upper_index: i32,
        liquidity: u128,
    ) -> Self {
        let summary = whirlpool.into();
        let tick_array_lower = summary.derive_tick_array(tick_lower_index);
        let tick_array_upper = summary.derive_tick_array(tick_upper_index);

        Self {
            position,
            position_mint,
            whirlpool: summary.address,
            tick_array_lower,
            tick_array_upper,
            tick_lower_index,
            tick_upper_index,
            liquidity,
        }
    }

    pub fn from_position(
        address: Pubkey,
        position: &WhirlpoolPosition,
        whirlpool: impl Into<WhirlpoolSummary>,
    ) -> Self {
        let summary = whirlpool.into();
        let tick_array_lower = summary.derive_tick_array(position.tick_lower_index);
        let tick_array_upper = summary.derive_tick_array(position.tick_upper_index);
        Self {
            position: address,
            position_mint: position.position_mint,
            whirlpool: summary.address,
            tick_array_lower,
            tick_array_upper,
            tick_lower_index: position.tick_lower_index,
            tick_upper_index: position.tick_upper_index,
            liquidity: position.liquidity,
        }
    }

    pub fn lower_sqrt_price(&self) -> u128 {
        sqrt_price_from_tick_index(self.tick_lower_index)
    }
    pub fn upper_sqrt_price(&self) -> u128 {
        sqrt_price_from_tick_index(self.tick_upper_index)
    }
}

pub mod derive {
    use super::*;
    /// Derive the address of a Whirlpool adapter account
    pub fn derive_margin_orca_config(
        airspace: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Pubkey {
        Pubkey::find_program_address(
            &[
                seeds::ORCA_ADAPTER_CONFIG,
                airspace.as_ref(),
                mint_a.as_ref(),
                mint_b.as_ref(),
            ],
            &MARGIN_ORCA_PROGRAM,
        )
        .0
    }

    /// Derive the address of the liquidity mint
    pub fn derive_position_mint(address: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[seeds::POSITION_NOTES, address.as_ref()],
            &MARGIN_ORCA_PROGRAM,
        )
        .0
    }

    pub fn derive_adapter_position_metadata(owner: &Pubkey, whirlpool_config: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                seeds::POSITION_METADATA,
                owner.as_ref(),
                whirlpool_config.as_ref(),
            ],
            &MARGIN_ORCA_PROGRAM,
        )
        .0
    }

    pub fn derive_margin_position(owner: &Pubkey, whirlpool_config: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                seeds::POSITION_NOTES,
                owner.as_ref(),
                whirlpool_config.as_ref(),
            ],
            &MARGIN_ORCA_PROGRAM,
        )
        .0
    }

    pub fn derive_whirlpool_mint(owner: &Pubkey, whirlpool: &Pubkey, seed: u64) -> Pubkey {
        Pubkey::find_program_address(
            &[
                seeds::POSITION_MINT,
                seed.to_le_bytes().as_ref(),
                owner.as_ref(),
                whirlpool.as_ref(),
            ],
            &MARGIN_ORCA_PROGRAM,
        )
        .0
    }
}

//! Whirlpool Swaps

use std::collections::{HashMap, HashSet};

use anchor_lang::ToAccountMetas;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use orca_whirlpool::state::Whirlpool;

use jet_instructions::{
    margin_swap::SwapAccounts,
    orca::{derive_tick_array, derive_whirlpool_oracle, start_tick_index},
};
use jet_margin_swap::accounts::OrcaWhirlpoolSwapPoolInfo;
use jet_program_common::programs::ORCA_WHIRLPOOL;
use jet_solana_client::rpc::{SolanaRpc, SolanaRpcExtra};

/// Accounts for a whirlpool swap
#[derive(Debug, Clone)]
pub struct WhirlpoolSwap {
    /// The address of the whirlpool
    pub whirlpool: Pubkey,

    /// TBD. Reserved for future use
    pub oracle: Pubkey,

    /// Tick arrays necessary for performing the swap
    pub tick_arrays: [Pubkey; 5],

    /// A vault
    pub token_vault_a: Pubkey,

    /// B vault
    pub token_vault_b: Pubkey,

    /// A mint
    pub token_a: Pubkey,

    /// B mint
    pub token_b: Pubkey,
}

impl WhirlpoolSwap {
    /// Get swap accounts for exchanging A for B
    pub fn swap_a_to_b(&self) -> impl SwapAccounts {
        WhirlpoolSwapAccounts {
            whirlpool: self.whirlpool,
            oracle: self.oracle,
            token_a: self.token_a,
            token_b: self.token_b,
            token_vault_a: self.token_vault_a,
            token_vault_b: self.token_vault_b,
            tick_arrays: [
                self.tick_arrays[2],
                self.tick_arrays[1],
                self.tick_arrays[0],
            ],
        }
    }

    /// Get swap accounts for exchanging B for A
    pub fn swap_b_to_a(&self) -> impl SwapAccounts {
        WhirlpoolSwapAccounts {
            whirlpool: self.whirlpool,
            oracle: self.oracle,
            token_a: self.token_a,
            token_b: self.token_b,
            token_vault_a: self.token_vault_a,
            token_vault_b: self.token_vault_b,
            tick_arrays: [
                self.tick_arrays[2],
                self.tick_arrays[3],
                self.tick_arrays[4],
            ],
        }
    }

    /// Get all swap pools that contain pairs of supported mints
    pub async fn get_pools(
        rpc: &(dyn SolanaRpc + 'static),
        supported_mints: &HashSet<Pubkey>,
    ) -> anyhow::Result<HashMap<(Pubkey, Pubkey), Self>> {
        Ok(rpc
            .find_anchor_accounts::<Whirlpool>()
            .await?
            .into_iter()
            .filter_map(|(address, whirlpool)| {
                if !supported_mints.contains(&whirlpool.token_mint_a)
                    || !supported_mints.contains(&whirlpool.token_mint_b)
                {
                    return None;
                }

                let ticks = [
                    start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, -2),
                    start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, -1),
                    start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 0),
                    start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 1),
                    start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 2),
                ];

                Some((
                    (whirlpool.token_mint_a, whirlpool.token_mint_b),
                    WhirlpoolSwap {
                        whirlpool: address,
                        oracle: derive_whirlpool_oracle(&address),
                        tick_arrays: [
                            derive_tick_array(&address, ticks[0], whirlpool.tick_spacing),
                            derive_tick_array(&address, ticks[1], whirlpool.tick_spacing),
                            derive_tick_array(&address, ticks[2], whirlpool.tick_spacing),
                            derive_tick_array(&address, ticks[3], whirlpool.tick_spacing),
                            derive_tick_array(&address, ticks[4], whirlpool.tick_spacing),
                        ],
                        token_vault_a: whirlpool.token_vault_a,
                        token_vault_b: whirlpool.token_vault_b,
                        token_a: whirlpool.token_mint_a,
                        token_b: whirlpool.token_mint_b,
                    },
                ))
            })
            .collect())
    }
}

struct WhirlpoolSwapAccounts {
    whirlpool: Pubkey,
    oracle: Pubkey,
    tick_arrays: [Pubkey; 3],
    token_vault_a: Pubkey,
    token_vault_b: Pubkey,
    token_a: Pubkey,
    token_b: Pubkey,
}

impl SwapAccounts for WhirlpoolSwapAccounts {
    fn to_account_meta(&self, _authority: Pubkey) -> Vec<AccountMeta> {
        OrcaWhirlpoolSwapPoolInfo {
            oracle: self.oracle,
            whirlpool: self.whirlpool,
            swap_program: ORCA_WHIRLPOOL,
            vault_a: self.token_vault_a,
            vault_b: self.token_vault_b,
            tick_array_0: self.tick_arrays[0],
            tick_array_1: self.tick_arrays[1],
            tick_array_2: self.tick_arrays[2],
        }
        .to_account_metas(None)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn pool_tokens(&self) -> (Pubkey, Pubkey) {
        (self.token_a, self.token_b)
    }

    fn route_type(&self) -> jet_margin_swap::SwapRouteIdentifier {
        jet_margin_swap::SwapRouteIdentifier::Whirlpool
    }
}

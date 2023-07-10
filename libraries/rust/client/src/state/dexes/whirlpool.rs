use std::sync::Arc;

use anchor_lang::ToAccountMetas;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use orca_whirlpool::state::Whirlpool;

use jet_environment::client_config::DexInfo;
use jet_instructions::{
    margin_swap::SwapAccounts,
    orca::{derive_tick_array, derive_whirlpool_oracle, start_tick_index},
};
use jet_margin_swap::accounts::OrcaWhirlpoolSwapPoolInfo;
use jet_program_common::programs::ORCA_WHIRLPOOL;
use jet_solana_client::rpc::SolanaRpcExtra;

use crate::{state::AccountStates, ClientResult};

use super::DexState;

pub async fn load_orca_whirlpools(states: &AccountStates, pools: &[DexInfo]) -> ClientResult<()> {
    let whirlpools = pools.iter().map(|w| w.address).collect::<Vec<_>>();

    log::debug!("loading whirlpools: {whirlpools:?}");

    let whirlpool_states = states
        .network
        .get_anchor_accounts::<Whirlpool>(&whirlpools)
        .await?;

    for (index, whirlpool) in whirlpool_states.into_iter().enumerate() {
        let address = whirlpools[index];
        let ticks = [
            start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, -2),
            start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, -1),
            start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 0),
            start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 1),
            start_tick_index(whirlpool.tick_current_index, whirlpool.tick_spacing, 2),
        ];

        let swap = WhirlpoolSwap {
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
        };

        let dex_state = DexState {
            program: pools[index].program,
            token_a: whirlpool.token_mint_a,
            token_b: whirlpool.token_mint_b,
            swap_a_to_b_accounts: Arc::new(swap.swap_a_to_b()),
            swap_b_to_a_accounts: Arc::new(swap.swap_b_to_a()),
        };

        states.set(&address, dex_state);
    }
    Ok(())
}

/// Accounts for a whirlpool swap
struct WhirlpoolSwap {
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
    fn swap_a_to_b(&self) -> impl SwapAccounts {
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
    fn swap_b_to_a(&self) -> impl SwapAccounts {
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

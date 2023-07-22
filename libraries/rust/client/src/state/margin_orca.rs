use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use orca_whirlpool::state::{Position as WhirlpoolPosition, Whirlpool};
use solana_sdk::pubkey::Pubkey;

use jet_instructions::margin_orca::derive;
use jet_margin::MarginAccount;
use jet_margin_orca::{PositionMetadata, WhirlpoolConfig};
use jet_solana_client::rpc::SolanaRpcExtra;

use super::AccountStates;
use crate::client::ClientResult;

/// Current state for a fixed term market
#[derive(Clone)]
pub struct WhirlpoolConfigState {
    pub config: WhirlpoolConfig,
    pub whirlpools: HashMap<Pubkey, Arc<Whirlpool>>,
}

#[derive(Clone)]
pub struct UserState {
    metadata: PositionMetadata,
    whirlpools: HashMap<Pubkey, Arc<Whirlpool>>,
    positions: HashMap<Pubkey, Arc<WhirlpoolPosition>>,
}

impl UserState {
    fn new(metadata: PositionMetadata) -> Self {
        Self {
            metadata,
            whirlpools: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    pub fn margin_account(&self) -> Pubkey {
        self.metadata.owner
    }

    pub fn whirlpool_config(&self) -> Pubkey {
        self.metadata.whirlpool_config
    }

    pub fn whirlpools(&self) -> impl IntoIterator<Item = Arc<Whirlpool>> {
        self.whirlpools.values().cloned().collect::<Vec<_>>()
    }

    pub fn positions(&self) -> impl IntoIterator<Item = Arc<WhirlpoolPosition>> {
        self.positions.values().cloned().collect::<Vec<_>>()
    }

    /// Get the addresses to use in a refresh position.
    /// This has to include the exact number of whirlpools in the user's positions
    pub fn addresses_for_refresh(&self) -> (HashSet<Pubkey>, HashSet<Pubkey>) {
        let mut whirlpools = HashSet::new();
        let mut positions = HashSet::new();

        for detail in self.metadata.positions() {
            whirlpools.insert(detail.whirlpool);
            positions.insert(detail.position);
        }

        (whirlpools, positions)
    }
}

impl Deref for UserState {
    type Target = PositionMetadata;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    sync_whirlpools(states).await?;
    sync_user_positions(states).await?;

    Ok(())
}

/// Sync latest state for all whirlpool pairs enabled in the adapter
pub async fn sync_whirlpools(states: &AccountStates) -> ClientResult<()> {
    let config_accounts = states
        .network
        .find_anchor_accounts::<WhirlpoolConfig>()
        .await?;
    let token_pairs = config_accounts
        .iter()
        .map(|(_, config)| (config.mint_a, config.mint_b))
        .collect::<HashSet<_>>();

    // Find all whirlpools, then filter them for ones where tokens pairs are registered
    let all_whirlpools = states.network.find_anchor_accounts::<Whirlpool>().await?;

    let supported_whirlpools = all_whirlpools
        .into_iter()
        .filter_map(|(address, pool)| {
            if token_pairs.contains(&(pool.token_mint_a, pool.token_mint_b)) {
                let pool = Arc::new(pool);
                // states.cache.set(&address, pool.clone());
                Some((address, pool))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    for (address, config) in config_accounts {
        // Group by supported token
        states.cache.set(
            &address,
            WhirlpoolConfigState {
                whirlpools: supported_whirlpools
                    .iter()
                    .filter_map(|(addr, pool)| {
                        if pool.token_mint_a == config.mint_a && pool.token_mint_b == config.mint_b
                        {
                            Some((*addr, pool.clone()))
                        } else {
                            None
                        }
                    })
                    .collect(),
                config,
            },
        );
    }

    Ok(())
}

/// Sync latest state for all user positions in a whirlpool pair
pub async fn sync_user_positions(states: &AccountStates) -> ClientResult<()> {
    let margin_accounts = states.addresses_of::<MarginAccount>();
    let configs = states.addresses_of::<WhirlpoolConfigState>();

    let orca_position_meta_addresses = margin_accounts
        .iter()
        .flat_map(|account| {
            configs
                .iter()
                .map(|config| derive::derive_adapter_position_metadata(account, config))
        })
        .collect::<Vec<_>>();

    let user_accounts = states
        .network
        .try_get_anchor_accounts::<PositionMetadata>(&orca_position_meta_addresses)
        .await?;

    for (address, user_state) in orca_position_meta_addresses.into_iter().zip(user_accounts) {
        if let Some(state) = user_state {
            states.cache.set(&address, UserState::new(state));
        }
    }

    sync_user_whirlpool_positions(states).await?;

    Ok(())
}

async fn sync_user_whirlpool_positions(states: &AccountStates) -> ClientResult<()> {
    let user_states = states.get_all::<UserState>();
    // Create hashsets of positions
    let position_addresses =
        HashSet::<Pubkey>::from_iter(user_states.iter().flat_map(|(_, state)| {
            let metadata: &PositionMetadata = state.deref();
            metadata.positions().into_iter().map(|p| p.position)
        }))
        .into_iter()
        .collect::<Vec<_>>();

    let positions = states
        .network
        .get_anchor_accounts::<WhirlpoolPosition>(&position_addresses)
        .await?
        .into_iter()
        .zip(position_addresses)
        .map(|(p, a)| (a, Arc::new(p)))
        .collect::<HashMap<_, _>>();

    // Iterate through user accounts and update them
    for user_state in user_states {
        let metadata: &PositionMetadata = user_state.1.deref();

        let mut new_state = UserState::new(metadata.clone());

        // Find whirlpools in the config
        // TODO: remove the expect() after testing
        let config = states
            .get::<WhirlpoolConfigState>(&metadata.whirlpool_config)
            .expect("No config");

        for detail in metadata.positions() {
            if let Some(position) = positions.get(&detail.position) {
                new_state
                    .positions
                    .insert(detail.position, position.clone());
            }
        }
        // TODO: should we filter them for pools a user explicitly interacts with?
        new_state.whirlpools = config.whirlpools.clone();

        states.cache.set(&user_state.0, new_state);
    }

    Ok(())
}

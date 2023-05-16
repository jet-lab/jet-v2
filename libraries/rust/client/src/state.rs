use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use jet_instructions::airspace::derive_airspace;
use jet_solana_client::rpc::SolanaRpc;
use solana_sdk::{address_lookup_table_account::AddressLookupTableAccount, pubkey::Pubkey};

use crate::{
    client::ClientResult,
    config::{DexInfo, JetAppConfig, TokenInfo},
    ClientError,
};

pub mod fixed_term;
pub mod margin;
pub mod margin_pool;
pub mod oracles;
pub mod spl_swap;
pub mod tokens;

/// A utility for synchronizing information about the current protocol state
/// with an active Solana network.
pub struct AccountStates {
    pub(crate) network: Arc<dyn SolanaRpc>,
    pub(crate) wallet: Pubkey,
    pub(crate) config: StateConfig,
    pub(crate) lookup_tables: LookupTableCache,
    cache: AccountCache,
}

impl AccountStates {
    /// Initialize an empty local state, which can synchronize data from the given interface
    pub fn new(
        network: Arc<dyn SolanaRpc>,
        wallet: Pubkey,
        app_config: JetAppConfig,
        airspace_seed: String,
    ) -> ClientResult<Self> {
        let airspace_config = app_config
            .airspaces
            .iter()
            .find(|entry| entry.name == airspace_seed)
            .ok_or_else(|| {
                ClientError::Unexpected(format!("no such airspace {airspace_seed} in app config"))
            })?;

        let config = StateConfig {
            airspace: derive_airspace(&airspace_seed),
            airspace_seed,
            airspace_lookup_registry_authority: airspace_config.lookup_registry_authority,
            tokens: airspace_config
                .tokens
                .clone()
                .iter()
                .filter_map(|name| app_config.tokens.iter().find(|t| t.name == *name))
                .cloned()
                .collect(),
            fixed_term_markets: airspace_config.fixed_term_markets.clone(),
            exchanges: app_config.exchanges.clone(),
        };

        log::debug!("loaded state config: {config:#?}");

        let cache = AccountCache::default();
        let lookup_tables = LookupTableCache::default();

        Ok(Self {
            config,
            wallet,
            network,
            cache,
            lookup_tables,
        })
    }

    pub async fn sync_all(&self) -> ClientResult<()> {
        self::oracles::sync(self).await?;
        self::spl_swap::sync(self).await?;
        self::margin_pool::sync(self).await?;
        self::fixed_term::sync(self).await?;
        self::margin::sync(self).await?;
        self::tokens::sync(self).await?;

        // Sync lookup tables

        Ok(())
    }

    pub fn token_info(&self, token: &Pubkey) -> ClientResult<TokenInfo> {
        self.config
            .tokens
            .iter()
            .find(|t| t.mint == *token)
            .cloned()
            .ok_or_else(|| ClientError::Unexpected(format!("missing token info for {token}")))
    }

    pub fn get_current_time(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }
}

impl std::ops::Deref for AccountStates {
    type Target = AccountCache;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

#[derive(Debug)]
pub struct StateConfig {
    pub airspace_seed: String,
    pub airspace: Pubkey,
    pub airspace_lookup_registry_authority: Option<Pubkey>,
    pub tokens: Vec<TokenInfo>,
    pub fixed_term_markets: Vec<Pubkey>,
    pub exchanges: Vec<DexInfo>,
}

type StoredStateObj = Arc<dyn Any + Send + Sync>;

#[derive(Default)]
pub struct AccountCache {
    states: Mutex<HashMap<TypeId, HashMap<Pubkey, Option<StoredStateObj>>>>,
}

impl AccountCache {
    pub fn addresses_of<T: Any>(&self) -> Vec<Pubkey> {
        let states = self.states.lock().unwrap();

        states
            .get(&TypeId::of::<T>())
            .map(|accounts| accounts.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn filter_addresses_of<T: Any>(
        &self,
        mut predicate: impl FnMut(&Pubkey, &T) -> bool,
    ) -> Vec<Pubkey> {
        let states = self.states.lock().unwrap();

        let accounts = match states
            .get(&TypeId::of::<T>())
            .map(|accounts| accounts.iter())
        {
            Some(accounts) => accounts,
            None => return vec![],
        };

        accounts
            .filter_map(|(address, data)| match data {
                Some(x) if predicate(address, x.downcast_ref().unwrap()) => Some(*address),
                _ => None,
            })
            .collect()
    }

    pub fn filter<T>(&self, mut predicate: impl FnMut(&Pubkey, &T) -> bool) -> Vec<(Pubkey, Arc<T>)>
    where
        T: Any + Send + Sync,
    {
        let states = self.states.lock().unwrap();

        let accounts = match states
            .get(&TypeId::of::<T>())
            .map(|accounts| accounts.iter())
        {
            Some(accounts) => accounts,
            None => return vec![],
        };

        accounts
            .filter_map(|(address, data)| match data {
                Some(x) if predicate(address, x.downcast_ref().unwrap()) => {
                    Some((*address, Arc::downcast(x.clone()).unwrap()))
                }
                _ => None,
            })
            .collect()
    }

    pub fn for_each<T: Any>(&self, mut action: impl FnMut(&Pubkey, &T)) {
        let states = self.states.lock().unwrap();
        if let Some(objects) = states.get(&TypeId::of::<T>()) {
            for (address, maybe_object) in objects {
                let maybe_state = maybe_object.as_ref().map(|o| o.downcast_ref().unwrap());

                if let Some(state) = maybe_state {
                    action(address, state)
                }
            }
        }
    }

    pub fn get_all<T: Any + Send + Sync>(&self) -> Vec<(Pubkey, Arc<T>)> {
        let mut result = vec![];

        let states = self.states.lock().unwrap();
        if let Some(objects) = states.get(&TypeId::of::<T>()) {
            for (address, maybe_object) in objects {
                if let Some(object) = maybe_object {
                    result.push((*address, Arc::downcast(object.clone()).unwrap()));
                }
            }
        }

        result
    }

    pub fn get<T: Any + Send + Sync>(&self, address: &Pubkey) -> Option<Arc<T>> {
        let states = self.states.lock().unwrap();

        states.get(&TypeId::of::<T>()).and_then(|accounts| {
            accounts
                .get(address)
                .cloned()
                .and_then(|account| account.map(|a| Arc::downcast(a).unwrap()))
        })
    }

    pub fn set<T: Any + Send + Sync>(&self, address: &Pubkey, data: T) {
        let type_id = TypeId::of::<T>();

        let mut states = self.states.lock().unwrap();

        let accounts = match states.get_mut(&type_id) {
            Some(accounts) => accounts,
            None => {
                states.insert(type_id, HashMap::new());
                states.get_mut(&type_id).unwrap()
            }
        };

        accounts.insert(*address, Some(Arc::new(data)));
    }

    pub fn register<T: Any + Send + Sync>(&self, address: &Pubkey) {
        let type_id = TypeId::of::<T>();

        let mut states = self.states.lock().unwrap();

        let accounts = match states.get_mut(&type_id) {
            Some(accounts) => accounts,
            None => {
                states.insert(type_id, HashMap::new());
                states.get_mut(&type_id).unwrap()
            }
        };

        if !accounts.contains_key(address) {
            accounts.insert(*address, None);
        }
    }
}

#[derive(Default)]
pub struct LookupTableCache {
    states: Mutex<HashMap<Pubkey, AddressLookupTableAccount>>,
}

impl LookupTableCache {
    pub fn get(&self, address: &Pubkey) -> Option<AddressLookupTableAccount> {
        let states = self.states.lock().unwrap();

        states.get(address).cloned()
    }

    pub fn set(&self, address: &Pubkey, data: AddressLookupTableAccount) {
        let mut states = self.states.lock().unwrap();

        states.insert(*address, data);
    }
}

use lookup_table_registry::RegistryAccount;
use lookup_table_registry_client::Entry;
use solana_address_lookup_table_program::state::AddressLookupTable;
use std::{collections::HashSet, sync::Arc};
use wasm_bindgen::prelude::*;

use bytemuck::Zeroable;

use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use jet_instructions::{
    fixed_term::derive as derive_fixed_term,
    margin::{
        derive_margin_account, derive_position_token_account, derive_token_config, MarginIxBuilder,
    },
    margin_pool::{derive_loan_account, derive_margin_pool, MarginPoolIxBuilder},
};
use jet_margin::{AccountPosition, MarginAccount, TokenAdmin, TokenConfig, TokenKind, TokenOracle};
use jet_margin_pool::{Amount, MarginPool, PoolAction};
use jet_program_common::Number128;
use jet_solana_client::rpc::SolanaRpcExtra;

use crate::{
    bail,
    client::{ClientError, ClientResult, ClientState},
    fixed_term::MarginAccountMarketClient,
    margin_orca::MarginAccountOrcaClient,
    margin_pool::MarginAccountPoolClient,
    state::{
        margin::load_margin_accounts,
        oracles::PriceOracleState,
        tokens::{Mint, TokenAccount},
    },
    swaps::MarginAccountSwapsClient,
    JetClient,
};

/// Client for interacting with the margin program
#[derive(Clone)]
pub struct MarginClient {
    client: Arc<ClientState>,
}

impl MarginClient {
    pub(crate) fn new(inner: Arc<ClientState>) -> Self {
        Self { client: inner }
    }

    /// Get the set of loaded margin accounts belonging to the current user
    pub fn accounts(&self) -> Vec<MarginAccountClient> {
        self.client
            .state()
            .filter_addresses_of::<MarginAccount>(|_, account| {
                account.owner == self.client.signer()
            })
            .into_iter()
            .map(|address| MarginAccountClient::new(self.client.clone(), address))
            .collect()
    }

    /// Sync all data related to margin accounting from the network
    pub async fn sync(&self) -> ClientResult<()> {
        crate::state::margin_pool::sync(self.client.state()).await?;
        crate::state::margin::sync(self.client.state()).await?;

        Ok(())
    }

    /// Create a new margin account for the current user
    ///
    /// The current client implementation is limited to creating maximum of 32 accounts per user.
    pub async fn create_account(&self) -> ClientResult<()> {
        let (index, (_, _)) = match self
            .get_possible_accounts()
            .await?
            .into_iter()
            .enumerate()
            .find(|(_, (_, account))| account.is_none())
        {
            None => {
                return Err(ClientError::Unexpected(
                    "user exceeded max accounts".to_string(),
                ))
            }
            Some(a) => a,
        };

        let builder =
            MarginIxBuilder::new(self.client.airspace(), self.client.signer(), index as u16);

        self.client.send(&builder.create_account()).await?;

        // Create an empty entry in the cache, so that a caller can immediately create a
        // client object to interact with the account (without having to resync first)
        let mut new_account = MarginAccount::zeroed();
        new_account.owner = self.client.signer();
        new_account.user_seed = (index as u16).to_le_bytes();

        self.client.state().set(&builder.address, new_account);

        Ok(())
    }

    async fn get_possible_accounts(&self) -> ClientResult<Vec<(Pubkey, Option<MarginAccount>)>> {
        // Currently limited to check a fixed set of accounts due to performance reasons,
        // as otherwise we would need to do an expensive `getProgramAccounts` to find them all.
        const MAX_DERIVED_ACCOUNTS_TO_CHECK: u16 = 32;

        let user = self.client.signer();
        let airspace = self.client.airspace();
        let possible_accounts = (0..MAX_DERIVED_ACCOUNTS_TO_CHECK)
            .map(|seed| derive_margin_account(&airspace, &user, seed))
            .collect::<Vec<_>>();

        let states = self
            .client
            .network
            .try_get_anchor_accounts::<MarginAccount>(&possible_accounts)
            .await?;

        Ok(possible_accounts.into_iter().zip(states).collect())
    }
}

/// Client for interacting with a specific margin account
#[derive(Clone)]
pub struct MarginAccountClient {
    pub(crate) client: Arc<ClientState>,
    pub(crate) address: Pubkey,
    pub(crate) builder: MarginIxBuilder,
}

impl MarginAccountClient {
    fn new(client: Arc<ClientState>, address: Pubkey) -> Self {
        let owner = client.signer();
        let builder = MarginIxBuilder::new_for_address(client.airspace(), address, owner);

        Self {
            client,
            address,
            builder,
        }
    }

    /// Get the root client object
    pub fn client(&self) -> JetClient {
        JetClient {
            client: self.client.clone(),
        }
    }

    pub fn state(&self) -> Arc<MarginAccount> {
        self.client.state().get(&self.address).unwrap()
    }

    /// The address of this account
    pub fn address(&self) -> Pubkey {
        self.address
    }

    /// the airspace the margin account is a part of
    pub fn airspace(&self) -> Pubkey {
        self.client.airspace()
    }

    /// The positions currently held by this account
    pub fn positions(&self) -> Vec<MarginPosition> {
        let list = self.positions_with_token_configs();

        list.into_iter()
            .map(|(config, position)| self.refreshed_position(&config, &position))
            .collect()
    }

    /// Get a client for using a margin pool with the current account
    pub fn pool(&self, token: &Pubkey) -> MarginAccountPoolClient {
        MarginAccountPoolClient::new(self.clone(), token)
    }

    /// Get a client for using swap pools with the current account
    pub fn swaps(&self) -> MarginAccountSwapsClient {
        MarginAccountSwapsClient::new(self.clone())
    }

    /// Get a client for using a fixed term market
    pub fn fixed_term(&self, market_address: &Pubkey) -> ClientResult<MarginAccountMarketClient> {
        MarginAccountMarketClient::from_address(self.clone(), market_address)
    }

    /// Get a client for using Orca for providing liquidity
    pub fn orca(&self, whirlpool: &Pubkey) -> ClientResult<MarginAccountOrcaClient> {
        MarginAccountOrcaClient::from_whirlpool(self.clone(), whirlpool)
    }

    /// Get the current balance of a token in the account
    pub fn balance(&self, token: &Pubkey) -> u64 {
        let address = get_associated_token_address(&self.address, token);
        self.client
            .state()
            .get::<TokenAccount>(&address)
            .map(|account| account.amount)
            .unwrap_or_default()
    }

    /// Resync the data for this account from the network
    pub async fn sync(&self) -> ClientResult<()> {
        load_margin_accounts(self.client.state(), &[self.address]).await
    }

    /// Send a transaction prefixed with refresh instructions for all positions
    pub async fn send_with_refresh(&self, instructions: &[Instruction]) -> ClientResult<()> {
        let mut ixs = self.instructions_for_refresh_positions()?;

        ixs.extend(instructions.iter().cloned());

        let lookup_tables = self.client.state().lookup_tables.get();

        self.client
            .send_with_lookup_tables(&ixs, &lookup_tables)
            .await
    }

    /// Close this margin account.
    ///
    /// This can be used to recover the SOL used as rent for the account data.
    ///
    /// The account must be empty (no registered positions) for it to be closed.
    pub async fn close(&self) -> ClientResult<()> {
        self.client.send(&self.builder.close_account()).await
    }

    /// Deposit tokens directly into the margin account as collateral
    ///
    /// These tokens are held in an ccount directly owned by this margin account, which can be
    /// used as collateral without being subject to contraints imposed by another contract
    /// (e.g. the margin lending pools).
    ///
    /// The tokens to deposit are transferred from the associated token account for the user,
    /// or can be provided explicitly.
    pub async fn deposit(
        &self,
        token: &Pubkey,
        amount: u64,
        source: Option<&Pubkey>,
    ) -> ClientResult<()> {
        let signer = self.client.signer();
        let mut ixns = vec![];

        let deposit_account = self.with_deposit_position(token, &mut ixns).await?;
        let deposit_source = source
            .cloned()
            .unwrap_or_else(|| get_associated_token_address(&signer, token));

        ixns.push(
            self.builder
                .transfer_deposit(signer, deposit_source, deposit_account, amount),
        );

        self.client.send(&ixns).await
    }

    /// Withdraw tokens directly from the margin account
    ///
    /// See [`deposit`]
    pub async fn withdraw(
        &self,
        token: &Pubkey,
        amount: u64,
        destination: Option<&Pubkey>,
    ) -> ClientResult<()> {
        let mut ixns = vec![];

        let deposit_account = get_associated_token_address(&self.address, token);
        let deposit_destination = match destination {
            Some(acc) => *acc,
            None => self.client.with_wallet_account(token, &mut ixns).await?,
        };

        ixns.push(self.builder.transfer_deposit(
            self.address,
            deposit_account,
            deposit_destination,
            amount,
        ));

        let deposit_balance = self
            .client
            .state()
            .get::<TokenAccount>(&deposit_account)
            .map(|state| state.amount)
            .unwrap_or_default();

        if deposit_balance == amount {
            ixns.push(self.builder.close_position(*token, deposit_account));
        }

        self.send_with_refresh(&ixns).await
    }

    /// Determine whether or not the currently loaded state for the margin account contains
    /// a position of the given token type.
    pub fn has_position(&self, token: &Pubkey) -> bool {
        self.state().positions().any(|p| p.token == *token)
    }

    /// Update lookup tables, creating new tables and adding addresses as necessary
    pub async fn update_lookup_tables(&self) -> ClientResult<()> {
        let just_created = self.init_lookup_registry().await?;

        if just_created {
            let slot = self.client.get_slot().await?;
            self.client.network.wait_for_slot(slot + 1).await?;
        }

        // Update existing tables, adding any new tables as necessary
        let mut accounts = HashSet::new();
        let state = self.client.state();
        for token in &state.config.tokens {
            let ata = get_associated_token_address(&self.address, &token.mint);
            accounts.insert(ata);
            let pool = MarginPoolIxBuilder::new(token.mint);
            accounts.insert(derive_position_token_account(
                &self.address,
                &pool.deposit_note_mint,
            ));
            accounts.insert(derive_loan_account(&self.address, &pool.loan_note_mint));
        }

        for market in self.client().fixed_term().markets() {
            // Term deposits and loans are ephemeral as they're based on a sequence number
            // Adding them would waste space
            let margin_user = derive_fixed_term::margin_user(&market.address, &self.address);
            accounts.insert(margin_user);
            accounts.insert(derive_fixed_term::user_claims(&margin_user));
            accounts.insert(derive_fixed_term::user_ticket_collateral(&margin_user));
            accounts.insert(derive_fixed_term::user_underlying_collateral(&margin_user));
        }

        let mut new_accounts = vec![];
        let registry_account = self
            .client
            .network
            .get_anchor_account::<RegistryAccount>(&self.builder.lookup_table_registry_address())
            .await?;
        let mut tables = Vec::with_capacity(registry_account.tables.len());

        // If the registry has no tables, add all accounts
        if registry_account.tables.is_empty() {
            new_accounts.extend(accounts);
        } else {
            for table in &registry_account.tables {
                if table.discriminator <= 1 {
                    // Deactivated or deleted
                    continue;
                }
                let lookup_table_account = self
                    .client()
                    .client
                    .network
                    .get_account(&table.table)
                    .await?;
                let Some(lookup_table_account) = lookup_table_account else {
                    continue;
                };

                let lookup_table = AddressLookupTable::deserialize(&lookup_table_account.data)?;

                let entry = lookup_table_registry_client::Entry {
                    discriminator: table.discriminator,
                    lookup_address: table.table,
                    addresses: lookup_table.addresses.to_vec(),
                };

                // Check if there are new accounts
                for address in &entry.addresses {
                    if !accounts.contains(address) {
                        new_accounts.push(*address);
                    }
                }

                tables.push(entry);
            }
        }

        if new_accounts.is_empty() {
            return Ok(());
        }

        // For now we are not picky about where addresses are stored, we use
        // lookup tables that have capacity.
        let mut append_instructions = vec![];
        let mut registry_index = 0;
        while !new_accounts.is_empty() {
            if tables.len() > registry_index {
                let entry = &tables[registry_index];
                registry_index += 1;
                let entry_capacity = 256usize.saturating_sub(entry.addresses.len());
                // Can fit in 25 addresses in a transaction
                let max_addresses = entry_capacity.min(25).min(new_accounts.len());
                let to_add = new_accounts.drain(..max_addresses).collect::<Vec<_>>();
                append_instructions.push(
                    self.builder
                        .append_to_lookup_table(entry.lookup_address, &to_add),
                );
            } else {
                let slot = self.client().client.get_slot().await?;
                let (table_ix, lookup_address) = self.builder.create_lookup_table(slot);
                append_instructions.push(table_ix);
                // Add the table to the registry, don't increment index
                tables.push(Entry {
                    discriminator: 2,
                    lookup_address,
                    addresses: vec![],
                });
            }
        }

        // Submit all instructions
        self.client.send_ordered(append_instructions).await
    }

    pub(crate) async fn with_deposit_position(
        &self,
        token: &Pubkey,
        ixns: &mut Vec<Instruction>,
    ) -> ClientResult<Pubkey> {
        let address = get_associated_token_address(&self.address, token);

        if !self.client.account_exists(&address).await? {
            ixns.push(create_associated_token_account(
                &self.client.signer(),
                &self.address,
                token,
                &spl_token::ID,
            ));
        }

        if !self.has_position(token) {
            ixns.push(self.builder.create_deposit_position(*token));
        }

        Ok(address)
    }

    pub(crate) fn token_config(&self, token: &Pubkey) -> ClientResult<TokenConfig> {
        let address = derive_token_config(&self.airspace(), token);

        self.client
            .state()
            .get::<TokenConfig>(&address)
            .map(|c| (*c).clone())
            .ok_or_else(|| ClientError::Unexpected(format!("no config found for token {token}")))
    }

    fn instructions_for_refresh_positions(&self) -> ClientResult<Vec<Instruction>> {
        let mut included = HashSet::new();
        let mut ixs = vec![];

        for position in self.state().positions() {
            if included.contains(&position.token) || position.balance == 0 {
                continue;
            }

            let token_config = self.token_config(&position.token)?;

            match position.adapter {
                id if id == Pubkey::default() => {
                    let oracle = match token_config.oracle() {
                        Some(TokenOracle::Pyth { price, .. }) => price,
                        _ => bail!("deposit position should have an oracle: {}", position.token),
                    };

                    ixs.push(
                        self.builder
                            .refresh_deposit_position(position.token, &oracle, true),
                    );
                }

                id if id == jet_margin_pool::ID => {
                    ixs.push(crate::margin_pool::instruction_for_refresh(
                        self,
                        &position.token,
                        &mut included,
                    )?);
                }

                id if id == jet_fixed_term::ID => {
                    ixs.push(crate::fixed_term::instruction_for_refresh(
                        self,
                        &position.token,
                        &mut included,
                    )?);
                }

                // id if id == jet_margin_orca::ID => {
                //     ixs.push(crate::margin_orca::instruction_for_refresh())
                // }
                address => {
                    return Err(ClientError::Unexpected(format!(
                        "position {} has unknown adapter {}",
                        position.address, address
                    )))
                }
            }

            included.insert(position.token);
        }

        Ok(ixs)
    }

    /// Update this local client state to reflect the current price information for held positions
    fn refreshed_position(
        &self,
        config: &TokenConfig,
        position: &AccountPosition,
    ) -> MarginPosition {
        let mut result = MarginPosition {
            token: position.token,
            underlying_token: config.underlying_mint,
            adapter: position.adapter,
            balance: position.balance,
            underlying_balance: position.balance,
            is_price_valid: false,
            value: Number128::ZERO,
            collateral_value: Number128::ZERO,
        };

        let underlying_config_address =
            derive_token_config(&self.client.airspace(), &config.underlying_mint);
        let underlying_config = match self
            .client
            .state()
            .get::<TokenConfig>(&underlying_config_address)
        {
            Some(config) => config,
            None => {
                log::error!(
                    "did not find config for position with underlying token {}",
                    config.underlying_mint
                );
                return result;
            }
        };

        let oracle = match underlying_config.admin {
            TokenAdmin::Margin { oracle } => {
                let TokenOracle::Pyth { price, .. } = oracle;
                price
            }
            _ => {
                log::error!(
                    "did not find oracle in config for position with underlying token {}",
                    config.underlying_mint
                );
                return result;
            }
        };

        let price_state = self
            .client
            .state()
            .get::<PriceOracleState>(&oracle)
            .unwrap();

        result.is_price_valid = price_state.is_valid;

        match position.adapter {
            id if id == Pubkey::default() => {
                self.refresh_deposit_position(&mut result, price_state.price)
            }

            id if id == jet_margin_pool::ID => {
                self.refresh_pool_position(&mut result, price_state.price)
            }

            id if id == jet_fixed_term::ID => {
                // FIXME:
                // Technically wrong, but should be a good enough approximation for now
                self.refresh_deposit_position(&mut result, price_state.price)
            }

            address => {
                log::error!(
                    "position {} has unknown adapter {}",
                    position.address,
                    address
                );
            }
        }

        result.collateral_value =
            result.value * Number128::from_decimal(position.value_modifier, -2);

        result
    }

    fn refresh_deposit_position(&self, position: &mut MarginPosition, price: Number128) {
        let mint = self
            .client
            .state()
            .get::<Mint>(&position.underlying_token)
            .unwrap();
        let balance_value = Number128::from_decimal(position.balance, -(mint.decimals as i32));

        position.value = balance_value * price;
    }

    fn refresh_pool_position(&self, position: &mut MarginPosition, price: Number128) {
        let mint = self
            .client
            .state()
            .get::<Mint>(&position.underlying_token)
            .unwrap();
        let config = self
            .client
            .state()
            .get::<TokenConfig>(&position.token)
            .unwrap();
        let pool_address = derive_margin_pool(&self.client.airspace(), &position.underlying_token);
        let pool = self
            .client
            .state()
            .get::<MarginPool>(&pool_address)
            .unwrap();

        let pool_action = match config.token_kind {
            TokenKind::Claim => PoolAction::Borrow,
            _ => PoolAction::Deposit,
        };

        let actual_current_balance_amount = pool
            .convert_amount(Amount::notes(position.balance), pool_action)
            .unwrap_or_default();

        let current_balance = Number128::from_decimal(
            actual_current_balance_amount.tokens,
            -(mint.decimals as i32),
        );

        position.underlying_balance = actual_current_balance_amount.tokens;
        position.value = current_balance * price;
    }

    fn positions_with_token_configs(&self) -> Vec<(TokenConfig, AccountPosition)> {
        self.state()
            .positions()
            .filter_map(|position| {
                let config_address = derive_token_config(&self.client.airspace(), &position.token);

                self.client
                    .state()
                    .get::<TokenConfig>(&config_address)
                    .map(|config| ((*config).clone(), *position))
            })
            .collect()
    }

    async fn init_lookup_registry(&self) -> ClientResult<bool> {
        let address = self.builder.lookup_table_registry_address();

        if self.client.account_exists(&address).await? {
            log::debug!(
                "Lookup registry {address} already exists for account {}",
                self.address
            );
            return Ok(false);
        }

        let ix = self.builder.init_lookup_registry();
        self.client.send(&ix).await?;

        Ok(true)
    }
}

/// Description for a position held by a margin account
#[wasm_bindgen]
#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub struct MarginPosition {
    /// The address the actual token being held in the position
    pub token: Pubkey,

    /// The underlying token, which this position is convertible into
    #[wasm_bindgen(js_name = underlyingToken)]
    pub underlying_token: Pubkey,

    /// The adapter program managing the position
    pub adapter: Pubkey,

    /// The position token balance
    pub balance: u64,

    /// The balance of the underlying token represented by this position
    #[wasm_bindgen(js_name = underlyingBalance)]
    pub underlying_balance: u64,

    /// Whether or not the current price provided by the oracle for this position is valid
    pub is_price_valid: bool,

    value: Number128,
    collateral_value: Number128,
}

#[wasm_bindgen]
impl MarginPosition {
    /// The approximate USD value for this position
    pub fn value(&self) -> f64 {
        self.value.as_f64()
    }

    /// The approximate USD value for this position when used as collateral.
    pub fn collateral_value(&self) -> f64 {
        self.collateral_value.as_f64()
    }
}

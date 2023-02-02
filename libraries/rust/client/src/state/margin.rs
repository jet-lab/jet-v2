use solana_sdk::pubkey::Pubkey;

use jet_instructions::margin::{derive_margin_account, derive_token_config};
use jet_margin::{MarginAccount, TokenAdmin, TokenConfig, TokenOracle};
use jet_margin_pool::MarginPool;
use jet_solana_client::{NetworkUserInterface, NetworkUserInterfaceExt};

use super::{fixed_term::MarketState, oracles::PriceOracleState, tokens::Mint, AccountStates};
use crate::{client::ClientResult, state::tokens};

/// Refresh state for all currently loaded margin accounts
pub async fn sync<I: NetworkUserInterface>(states: &AccountStates<I>) -> ClientResult<I, ()> {
    sync_configs(states).await?;
    sync_margin_accounts(states).await?;

    Ok(())
}

/// Reload all state for the margin configurations
pub async fn sync_configs<I: NetworkUserInterface>(
    states: &AccountStates<I>,
) -> ClientResult<I, ()> {
    let mut tokens = states
        .config
        .tokens
        .iter()
        .map(|info| info.mint)
        .collect::<Vec<_>>();

    // Tokens for pool positions
    states.for_each(|_, pool: &MarginPool| {
        tokens.push(pool.deposit_note_mint);
        tokens.push(pool.loan_note_mint);
    });

    // Tokens from fixed term markets
    states.for_each(|_, market_state: &MarketState| {
        tokens.push(market_state.market.claims_mint);
        tokens.push(market_state.market.ticket_collateral_mint);
        tokens.push(market_state.market.ticket_mint);
    });

    // derive all the config addresses
    let configs = tokens
        .iter()
        .map(|token| derive_token_config(&states.config.airspace, token))
        .collect::<Vec<_>>();

    let accounts = states
        .network
        .get_anchor_accounts::<TokenConfig>(&configs)
        .await?;

    for (index, account) in accounts.into_iter().enumerate() {
        let address = configs[index];

        match account {
            None => {
                log::warn!(
                    "missing expected margin token config for token {}",
                    tokens[index]
                )
            }

            Some(config) => {
                states.register::<Mint>(&config.mint);
                states.register::<Mint>(&config.underlying_mint);

                if let TokenAdmin::Margin { oracle } = &config.admin {
                    let TokenOracle::Pyth { price, .. } = oracle;
                    states.register::<PriceOracleState>(price);
                }

                states.set(&address, config);
            }
        }
    }

    Ok(())
}

/// Sync all latest state for all previouly loaded margin accounts
pub async fn sync_margin_accounts<I: NetworkUserInterface>(
    states: &AccountStates<I>,
) -> ClientResult<I, ()> {
    let addresses = states.cache.addresses_of::<MarginAccount>();

    load_margin_accounts(states, &addresses).await
}

/// Load state for the given list of margin accounts
pub async fn load_margin_accounts<I: NetworkUserInterface>(
    states: &AccountStates<I>,
    addresses: &[Pubkey],
) -> ClientResult<I, ()> {
    let accounts = states
        .network
        .get_anchor_accounts::<MarginAccount>(addresses)
        .await?;

    let mut positions = vec![];

    for (address, account) in addresses.iter().zip(accounts) {
        if let Some(account) = account {
            positions.extend(account.positions().map(|p| p.address));
            states.cache.set(address, account);
        }
    }

    tokens::load_accounts(states, &positions).await?;

    Ok(())
}

/// Load the state for the margin accounts associated with the current connected wallet
///
/// This is currently limited to only finding the first 32 addresses associated
/// with the user based on the account seed value.
pub async fn load_user_margin_accounts<I: NetworkUserInterface>(
    states: &AccountStates<I>,
) -> ClientResult<I, ()> {
    // Currently limited to check a fixed set of accounts due to performance reasons,
    // as otherwise we would need to do an expensive `getProgramAccounts` to find them all.
    const MAX_DERIVED_ACCOUNTS_TO_CHECK: u16 = 32;

    let user = states.network.signer();
    let airspace = states.config.airspace;
    let possible_accounts = (0..MAX_DERIVED_ACCOUNTS_TO_CHECK)
        .map(|seed| derive_margin_account(&airspace, &user, seed))
        .collect::<Vec<_>>();

    let maybe_accounts = states
        .network
        .get_anchor_accounts::<MarginAccount>(&possible_accounts)
        .await?;

    let mut positions = vec![];

    for (address, maybe_account) in possible_accounts.into_iter().zip(maybe_accounts) {
        if let Some(account) = maybe_account {
            positions.extend(account.positions().map(|p| p.address));
            states.cache.set(&address, account);
        }
    }

    tokens::load_accounts(states, &positions).await?;

    Ok(())
}

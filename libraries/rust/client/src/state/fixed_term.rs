use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Range,
    sync::Arc,
};

use anchor_lang::AccountDeserialize;
use solana_sdk::{account::Account, pubkey::Pubkey};

use agnostic_orderbook::state::{
    critbit::{Node, NodeHandle, Slab},
    get_side_from_order_id, AccountTag, Side,
};

use jet_fixed_term::{
    control::state::Market,
    margin::state::{MarginUser, TermLoan},
    orderbook::state::{CallbackInfo, OrderTag},
    tickets::state::TermDeposit,
};
use jet_instructions::fixed_term::derive;
use jet_margin::MarginAccount;
use jet_solana_client::rpc::SolanaRpcExtra;

use super::AccountStates;
use crate::{
    client::ClientResult,
    fixed_term::util::{f64_to_price, price_to_rate, ui_price},
};

pub type FixedTermUser = MarginUser;

/// Current state for a fixed term market
pub struct MarketState {
    pub market: Market,
    pub asks: BTreeSet<OrderEntry>,
    pub bids: BTreeSet<OrderEntry>,
}

#[derive(Default, Debug, Clone)]
pub struct OrderEntry {
    pub order_id: u128,
    pub order_tag: OrderTag,
    pub owner: Pubkey,
    pub price: f64,
    pub base_token_amount: u64,
    tenor: u64,
}

impl OrderEntry {
    pub fn side(&self) -> Side {
        get_side_from_order_id(self.order_id)
    }

    pub fn rate(&self) -> u64 {
        price_to_rate(f64_to_price(self.price), self.tenor)
    }
}

impl PartialOrd for OrderEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order_id.partial_cmp(&other.order_id)
    }
}

impl PartialEq for OrderEntry {
    fn eq(&self, other: &Self) -> bool {
        self.order_id.eq(&other.order_id)
    }
}

impl Eq for OrderEntry {}

impl Ord for OrderEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order_id.cmp(&other.order_id)
    }
}

pub struct UserState {
    state: MarginUser,
    loans: BTreeMap<u64, Arc<TermLoan>>,
    deposits: BTreeMap<u64, Arc<TermDeposit>>,
}

impl UserState {
    fn new(state: MarginUser) -> Self {
        Self {
            state,
            loans: BTreeMap::new(),
            deposits: BTreeMap::new(),
        }
    }

    pub fn margin_account(&self) -> Pubkey {
        self.state.margin_account
    }

    pub fn market(&self) -> Pubkey {
        self.state.market
    }

    pub fn loans(&self) -> impl IntoIterator<Item = Arc<TermLoan>> {
        self.loans.values().cloned().collect::<Vec<_>>()
    }

    pub fn deposits(&self) -> impl IntoIterator<Item = Arc<TermDeposit>> {
        self.deposits.values().cloned().collect::<Vec<_>>()
    }
}

impl std::ops::Deref for UserState {
    type Target = MarginUser;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    sync_markets(states).await?;
    sync_user_accounts(states).await?;

    Ok(())
}

/// Sync latest state for all fixed term lending markets
pub async fn sync_markets(states: &AccountStates) -> ClientResult<()> {
    let manager_accounts = states
        .network
        .try_get_anchor_accounts::<Market>(&states.config.fixed_term_markets)
        .await?;

    let (ask_keys, bid_keys): (Vec<_>, Vec<_>) = manager_accounts
        .iter()
        .filter_map(|m| m.map(|m| (m.asks, m.bids)))
        .unzip();

    let ask_accounts = states.network.get_accounts_all(&ask_keys).await?;
    let bid_accounts = states.network.get_accounts_all(&bid_keys).await?;

    let mut asks: HashMap<_, _> = HashMap::from_iter(ask_keys.into_iter().zip(ask_accounts));
    let mut bids: HashMap<_, _> = HashMap::from_iter(bid_keys.into_iter().zip(bid_accounts));

    for (address, market) in states
        .config
        .fixed_term_markets
        .iter()
        .cloned()
        .zip(manager_accounts)
    {
        if let Some(market) = market {
            let Some(asks_acc) = asks.remove(&market.asks).unwrap() else {
                log::error!("missing asks account for market {address}");
                continue;
            };

            let Some(bids_acc) = bids.remove(&market.bids).unwrap() else {
                log::error!("missing bids account for market {address}");
                continue;
            };

            let Ok((asks, bids)) = parse_bid_asks(&address, market.borrow_tenor, asks_acc, bids_acc) else {
                continue;
            };

            states
                .cache
                .set(&address, MarketState { market, asks, bids });
        }
    }

    Ok(())
}

/// Sync latest state for all fixed term user data
///
/// The user data for all loaded margin accounts are fetched
pub async fn sync_user_accounts(states: &AccountStates) -> ClientResult<()> {
    let margin_accounts = states.addresses_of::<MarginAccount>();
    let markets = states.addresses_of::<MarketState>();

    let ft_user_accounts = margin_accounts
        .iter()
        .flat_map(|account| {
            markets
                .iter()
                .map(|market| derive::margin_user(market, account))
        })
        .collect::<Vec<_>>();

    let user_accounts = states
        .network
        .try_get_anchor_accounts::<FixedTermUser>(&ft_user_accounts)
        .await?;

    for (address, user_state) in ft_user_accounts.into_iter().zip(user_accounts) {
        if let Some(state) = user_state {
            states.cache.set(&address, UserState::new(state));
        }
    }

    sync_user_debt_assets(states).await?;

    Ok(())
}

async fn sync_user_debt_assets(states: &AccountStates) -> ClientResult<()> {
    let loans: Vec<Arc<TermLoan>> = load_user_positions(
        states,
        |state| state.debt().active_loans(),
        |user, state, seqno| derive::term_loan(&state.market, user, seqno),
    )
    .await?;

    let deposits: Vec<Arc<TermDeposit>> = load_user_positions(
        states,
        |state| state.assets().active_deposits(),
        |_, state, seqno| derive::term_deposit(&state.market, &state.margin_account, seqno),
    )
    .await?;

    let user_states = states.get_all::<UserState>();
    let mut user_account_map = HashMap::new();
    user_states.iter().for_each(|s| {
        user_account_map.insert(s.1.margin_account(), s.0);
    });

    let mut user_updates = HashMap::new();

    for (user, root) in &user_states {
        if !user_updates.contains_key(user) {
            user_updates.insert(*user, UserState::new(root.state.clone()));
        }
    }

    for loan in loans {
        let state = user_updates.get_mut(&loan.margin_user).unwrap();
        state.loans.insert(loan.sequence_number, loan.clone());
    }

    for deposit in deposits {
        let user = user_account_map.get(&deposit.owner).unwrap();
        let state = user_updates.get_mut(user).unwrap();
        state
            .deposits
            .insert(deposit.sequence_number, deposit.clone());
    }

    for (addr, new_state) in user_updates {
        states.cache.set(&addr, new_state);
    }

    Ok(())
}

async fn load_user_positions<T, FR, FD>(
    states: &AccountStates,
    range: FR,
    derive_addr: FD,
) -> ClientResult<Vec<Arc<T>>>
where
    T: Any + Send + Sync + AccountDeserialize,
    FR: Fn(&MarginUser) -> Range<u64>,
    FD: Fn(&Pubkey, &MarginUser, u64) -> Pubkey,
{
    let user_states = states.get_all::<UserState>();

    let (users, addresses): (Vec<_>, Vec<_>) = user_states
        .iter()
        .flat_map(|(user_addr, root)| {
            range(&root.state).map(|seqno| (*user_addr, derive_addr(user_addr, &root.state, seqno)))
        })
        .unzip();

    let accounts_data = states
        .network
        .try_get_anchor_accounts::<T>(&addresses)
        .await?;

    Ok(accounts_data.into_iter().enumerate().filter_map(|(idx, maybe_data)| {
        let Some(data) = maybe_data else {
            log::warn!("missing expected account {} ({}), for market user {}", addresses[idx], std::any::type_name::<T>(), users[idx]);
            return None;
        };

        states.cache.set(&addresses[idx], data);
        states.cache.get(&addresses[idx])
    }).collect())
}

fn parse_bid_asks(
    market: &Pubkey,
    tenor: u64,
    mut asks: Account,
    mut bids: Account,
) -> Result<(BTreeSet<OrderEntry>, BTreeSet<OrderEntry>), ()> {
    let ask_slab = match Slab::<CallbackInfo>::from_buffer(&mut asks.data, AccountTag::Asks) {
        Ok(slab) => slab,
        Err(e) => {
            log::error!("could not load asks account for market {market}: {e:?}");
            return Err(());
        }
    };
    let bid_slab = match Slab::<CallbackInfo>::from_buffer(&mut bids.data, AccountTag::Bids) {
        Ok(slab) => slab,
        Err(e) => {
            log::error!("could not load bids account for market {market}: {e:?}");
            return Err(());
        }
    };

    fn walk_nodes(
        tenor: u64,
        slab: &Slab<CallbackInfo>,
        root: NodeHandle,
        output: &mut BTreeSet<OrderEntry>,
    ) {
        match Node::from_handle(root) {
            Node::Inner => {
                let node = slab.inner_nodes[(!root) as usize];
                walk_nodes(tenor, slab, node.children[0], output);
                walk_nodes(tenor, slab, node.children[1], output);
            }

            Node::Leaf => {
                let node = slab.leaf_nodes[root as usize];
                let ft_info = slab.get_callback_info(root);

                output.insert(OrderEntry {
                    order_id: node.key,
                    order_tag: ft_info.order_tag(),
                    owner: ft_info.owner(),
                    price: ui_price(node.price()),
                    base_token_amount: node.base_quantity,
                    tenor,
                });
            }
        }
    }

    let mut ask_entries = BTreeSet::new();
    let mut bid_entries = BTreeSet::new();

    if let Some(ask_root) = ask_slab.root() {
        walk_nodes(tenor, &ask_slab, ask_root, &mut ask_entries);
    }

    if let Some(bid_root) = bid_slab.root() {
        walk_nodes(tenor, &bid_slab, bid_root, &mut bid_entries);
    }

    Ok((ask_entries, bid_entries))
}

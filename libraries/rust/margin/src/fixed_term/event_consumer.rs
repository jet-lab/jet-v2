use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use agnostic_orderbook::state::{
    event_queue::{EventQueue, EventRef, FillEventRef, OutEventRef},
    AccountTag,
};
use anchor_lang::AccountDeserialize;
use futures::{future::join_all, lock::Mutex as AsyncMutex};
use jet_solana_client::rpc::AccountFilter;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, packet::PACKET_DATA_SIZE, pubkey::Pubkey,
    signer::Signer, transaction::Transaction,
};
use thiserror::Error;

use jet_fixed_term::{
    control::state::Market,
    margin::state::MarginUser,
    orderbook::state::{
        CallbackFlags, CallbackInfo, MarginCallbackInfo, SignerCallbackInfo, UserCallbackInfo,
    },
};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use tracing::instrument;

use crate::util::no_dupe_queue::AsyncNoDupeQueue;

use super::{FixedTermIxBuilder, OwnedEventQueue};

const MAX_EVENTS_PER_TX: usize = 8;

#[derive(Error, Debug)]
pub enum EventConsumerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] anyhow::Error),

    #[error("the address is not a market account: {0}")]
    InvalidMarketAccount(Pubkey),

    #[error("the event queue is not readable: {0}")]
    InvalidEventQueue(Pubkey),

    #[error("failed to fetch user: {0}")]
    InvalidUserKey(Pubkey),
}

/// Utility for running consume-events for fixed term markets
pub struct EventConsumer {
    rpc: Arc<dyn SolanaRpcClient>,
    markets: Mutex<HashMap<Pubkey, Arc<AsyncMutex<MarketState>>>>,
}

/// does not guarantee successful downloads, some may be omitted
pub async fn download_markets(
    rpc: &dyn SolanaRpcClient,
    addresses: &[Pubkey],
) -> Result<Vec<Market>, EventConsumerError> {
    let markets = rpc.get_multiple_accounts(addresses).await?;
    let mut structures = vec![];
    for (address, market) in addresses.iter().zip(markets) {
        if let Some(data) = market.map(|m| m.data) {
            structures.push(
                Market::try_deserialize(&mut &data[..])
                    .map_err(|_| EventConsumerError::InvalidMarketAccount(*address))?,
            );
        } else {
            tracing::warn!("missing market {address}");
        }
    }

    Ok(structures)
}

impl EventConsumer {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            rpc,
            markets: Mutex::new(HashMap::new()),
        }
    }

    /// Load fixed term markets to have their events consumed
    /// Assumes there is no one listening for margin accounts to settle
    pub async fn load_markets(&self, addresses: &[Pubkey]) -> Result<(), EventConsumerError> {
        for market in download_markets(self.rpc.as_ref(), addresses).await? {
            self.insert_market(market, None);
        }

        Ok(())
    }

    /// Insert fixed term market to enable it to have its events consumed
    /// optionally include a sink representing a listener for accounts that need to be settled
    pub fn insert_market(
        &self,
        market: Market,
        margin_account_settlement_sink: Option<AsyncNoDupeQueue<Pubkey>>,
    ) {
        let builder = FixedTermIxBuilder::new_from_state(self.rpc.payer().pubkey(), &market);
        self.markets.lock().unwrap().insert(
            builder.market(),
            Arc::new(AsyncMutex::new(MarketState {
                market_address: builder.market(),
                market,
                queue: Vec::new(),
                users: HashMap::new(),
                builder,
                margin_accounts_to_settle: margin_account_settlement_sink,
            })),
        );
    }

    /// Start a loop to continuously consume events. Never returns. Logs errors
    pub async fn sync_and_consume_forever(&self, targets: &[Pubkey], delay: Duration) {
        loop {
            if let Err(e) = self.sync_and_consume_all(targets).await {
                tracing::error!("Error while consuming events: {e:?}");
            }
            tokio::time::sleep(delay).await;
        }
    }

    pub async fn sync_and_consume_all(&self, targets: &[Pubkey]) -> Result<(), EventConsumerError> {
        self.sync_queues().await?;
        while self.total_pending_events(targets).await? > 0 {
            self.sync_users().await?;
            self.consume().await?;
            self.sync_queues().await?;
        }

        Ok(())
    }

    /// Sync state for all users in the market
    pub async fn sync_users(&self) -> Result<(), EventConsumerError> {
        tracing::trace!("beginning user sync");

        let accounts = self
            .rpc
            .get_program_accounts(
                &jet_fixed_term::ID,
                vec![AccountFilter::DataSize(
                    8 + std::mem::size_of::<MarginUser>(),
                )],
            )
            .await?;

        for (address, user) in accounts {
            let structure = match MarginUser::try_deserialize(&mut &user.data[..]) {
                Ok(u) => u,
                Err(e) => {
                    tracing::info!("failed to deserialize margin user {address}: {e}");
                    continue;
                }
            };

            if let Some(state) = self.get_market(&structure.market) {
                let mut state = state.lock().await;

                if state.users.insert(address, structure.clone()).is_none() {
                    tracing::trace!(
                        "found user {address} (owned by {}) in market {}",
                        structure.margin_account,
                        structure.market
                    );
                }
            }
        }

        if tracing::enabled!(tracing::Level::TRACE) {
            for (market, state) in self.markets() {
                let state = state.lock().await;
                tracing::trace!(?market, "sync {} total users", state.users.len());
            }
        }

        Ok(())
    }

    fn get_market(&self, market: &Pubkey) -> Option<Arc<AsyncMutex<MarketState>>> {
        self.markets.lock().unwrap().get(market).cloned()
    }

    fn markets(&self) -> impl Iterator<Item = (Pubkey, Arc<AsyncMutex<MarketState>>)> {
        self.markets
            .lock()
            .unwrap()
            .iter()
            .map(|(x, y)| (*x, y.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Sync the event queues
    pub async fn sync_queues(&self) -> Result<(), EventConsumerError> {
        tracing::trace!("sync event queues");

        let (markets, addresses): (Vec<_>, Vec<_>) =
            join_all(self.markets().map(|(addr, state)| async move {
                let state = state.lock().await;
                (addr, state.market.event_queue)
            }))
            .await
            .into_iter()
            .unzip();

        let accounts = self.rpc.get_multiple_accounts(&addresses).await?;

        for (market, account) in markets.into_iter().zip(accounts) {
            let mkt = self.get_market(&market).unwrap();
            let mut market_state = mkt.lock().await;

            if let Some(account) = account {
                market_state.queue = account.data;

                tracing::trace!(?market, "sync queue {}", market_state.market.event_queue);
            } else {
                tracing::error!(?market, "queue account missing");
            }
        }

        Ok(())
    }

    /// Consume events by sending a transaction to each market
    pub async fn consume(&self) -> Result<(), EventConsumerError> {
        tracing::trace!("trying to consume events");

        let tasks = self
            .markets
            .lock()
            .unwrap()
            .iter()
            .map(|(address, state)| (*address, state.clone()))
            .map(|(address, state)| async move {
                let mut state = state.lock().await;
                state
                    .consume_next(&*self.rpc)
                    .await
                    .map_err(|e| (address, e))
            })
            .collect::<Vec<_>>();

        let results = futures::future::join_all(tasks).await;

        for result in &results {
            if let Err((market, e)) = result {
                tracing::error!(?market, "failed consuming events because: {e}",);
            }
        }

        if tracing::enabled!(tracing::Level::DEBUG) {
            let success = results.iter().filter(|r| r.is_ok()).count();
            let failed = results.len() - success;

            tracing::debug!(success, failed, "attempted consume events")
        }

        Ok(())
    }

    /// Count the events waiting to be consumed in a market
    pub async fn pending_events(&self, market: &Pubkey) -> Result<usize, EventConsumerError> {
        match self.get_market(market) {
            Some(state) => state.lock().await.pending_events(),
            None => Ok(0),
        }
    }

    /// Count the total events waiting to be consumed in all provided market
    pub async fn total_pending_events(
        &self,
        targets: &[Pubkey],
    ) -> Result<usize, EventConsumerError> {
        Ok(join_all(
            targets
                .iter()
                .map(|market| async { self.pending_events(market).await.unwrap() }),
        )
        .await
        .into_iter()
        .sum::<usize>())
    }
}

#[derive(Clone)]
struct MarketState {
    market_address: Pubkey,
    market: Market,
    queue: Vec<u8>,
    users: HashMap<Pubkey, MarginUser>,
    builder: FixedTermIxBuilder,
    /// send margin accounts here once they need to be settled
    margin_accounts_to_settle: Option<AsyncNoDupeQueue<Pubkey>>,
}

impl MarketState {
    #[instrument(skip(self, rpc), fields(market = %self.market_address))]
    async fn consume_next(&mut self, rpc: &dyn SolanaRpcClient) -> Result<(), EventConsumerError> {
        let mut queue: OwnedEventQueue = self.queue.clone().into();

        let payer = rpc.payer().pubkey();
        let payer_key = rpc.payer();
        let recent_blockhash = rpc.get_latest_blockhash().await?;
        let mut consume_params = vec![];
        let mut consume_tx = Transaction::default();
        let mut margin_accounts_to_settle = Vec::new();

        tracing::debug!(
            "processing queue of length {}",
            queue.inner().unwrap().len()
        );

        for event in queue
            .inner()
            .map_err(|_| EventConsumerError::InvalidEventQueue(self.market.event_queue))?
            .iter()
        {
            let mut seed = make_seed();
            match event {
                EventRef::Fill(FillEventRef {
                    maker_callback_info,
                    taker_callback_info,
                    ..
                }) => {
                    let maker_user_callback_info = UserCallbackInfo::from(*maker_callback_info);
                    tracing::trace!(
                        "prepare to handle fill event: {:#?}",
                        &maker_user_callback_info
                    );

                    let fill_accounts = match maker_user_callback_info {
                        UserCallbackInfo::Margin(info) => {
                            margin_accounts_to_settle.push(info.margin_account);

                            FillAccounts {
                                user_accounts: self.margin_fill_accounts(&mut seed, &info)?,
                                maker_queue: maybe_adapter!(info),
                                taker_queue: taker_callback_info.adapter(),
                            }
                        }
                        UserCallbackInfo::Signer(info) => FillAccounts {
                            user_accounts: self.signer_fill_accounts(&seed, &info),
                            maker_queue: maybe_adapter!(info),
                            taker_queue: taker_callback_info.adapter(),
                        },
                    };

                    tracing::trace!("add accounts for fill: {:#?}", &fill_accounts);
                    consume_params.push(EventAccounts::Fill(fill_accounts))
                }
                EventRef::Out(OutEventRef { callback_info, .. }) => {
                    let user_callback_info = UserCallbackInfo::from(*callback_info);
                    tracing::trace!("prepare to handle out event: {:#?}", &user_callback_info);

                    let out_accounts = match user_callback_info {
                        UserCallbackInfo::Margin(info) => {
                            margin_accounts_to_settle.push(info.margin_account);
                            OutAccounts {
                                out_account: info.margin_user,
                                user_queue: maybe_adapter!(info),
                            }
                        }
                        UserCallbackInfo::Signer(info) => OutAccounts {
                            out_account: info.token_account,
                            user_queue: maybe_adapter!(info),
                        },
                    };

                    tracing::trace!("add accounts for out: {:#?}", &out_accounts);
                    consume_params.push(EventAccounts::Out(out_accounts))
                }
            }

            let consume_ix = [
                ComputeBudgetInstruction::set_compute_unit_limit(800_000),
                self.builder.consume_events(&seed, &consume_params),
            ];
            let next_tx = Transaction::new_signed_with_payer(
                &consume_ix,
                Some(&payer),
                &[payer_key],
                recent_blockhash,
            );

            let Ok(tx_serialized) = bincode::serialize(&next_tx) else {
                panic!("producing unserializable transaction: {next_tx:?}");
            };

            if tx_serialized.len() >= PACKET_DATA_SIZE || consume_params.len() == MAX_EVENTS_PER_TX
            {
                break;
            }

            consume_tx = next_tx;
        }

        if consume_params.is_empty() {
            tracing::trace!("no events to consume");
            return Ok(());
        }

        self.pop_events(consume_params.len())?;

        rpc.send_and_confirm_transaction(&consume_tx).await?;
        if let Some(sink) = self.margin_accounts_to_settle.as_ref() {
            sink.push_many(margin_accounts_to_settle).await;
        }

        Ok(())
    }

    fn margin_fill_accounts(
        &mut self,
        seed: &mut Vec<u8>,
        info: &MarginCallbackInfo,
    ) -> Result<UserFillAccounts, EventConsumerError> {
        let term_account = if info.flags.contains(CallbackFlags::AUTO_STAKE) {
            // If auto-stake is enabled for lending, then consuming the event
            // requires passing in the right address for the `TermDeposit` account
            // to be created now that the loan has been filled
            //
            // This line mutates the `MarginUser` to allow the proper seed to be derived on sub-
            // sequent calls, but due to incomplete information user assets will not be properly
            // accounted for until state is re-synced
            *seed = self
                .users
                .get_mut(&info.margin_user)
                .ok_or(EventConsumerError::InvalidUserKey(info.margin_user))?
                .maker_fill_lend_order(true, 1)
                .map_err(|_| EventConsumerError::InvalidUserKey(info.margin_user))?
                .to_le_bytes()
                .to_vec();

            let deposit = self.builder.term_deposit_key(&info.margin_account, seed);
            tracing::debug!(
                owner = ?info.margin_account,
                "prepare to fill auto-stake for lender to: {}",
                deposit
            );
            Some(deposit)
        } else if info.flags.contains(CallbackFlags::NEW_DEBT) {
            // If this fill is issuing debt, then consuming requires passing in
            // the address for the `TermLoan` account to be created for tracking
            // the user debt
            if let Some(maker_user) = self.users.get_mut(&info.margin_user) {
                // In this case, the maker is using a margin account, so we
                // derive the new `TermLoan` account based on the debt sequence
                // number in the account state
                *seed = maker_user
                    .debt()
                    .next_new_loan_seqno()
                    .to_le_bytes()
                    .to_vec();

                let loan_account = Some(self.builder.term_loan_key(&info.margin_account, seed));

                tracing::debug!(
                    owner = ?info.margin_account,
                    "prepare to fill debt for borrower to: {}",
                    loan_account.as_ref().unwrap()
                );
                loan_account
            } else {
                tracing::error!(
                    "unexpected debt fill with non-margin user account: {}",
                    info.margin_user
                );
                None
            }
        } else {
            None
        };

        Ok(UserFillAccounts::Margin(MarginFillAccounts {
            margin_user: info.margin_user,
            term_account,
        }))
    }
    fn signer_fill_accounts(&mut self, seed: &[u8], info: &SignerCallbackInfo) -> UserFillAccounts {
        let fill = if info.flags.contains(CallbackFlags::AUTO_STAKE) {
            let deposit = self.builder.term_deposit_key(&info.signer, seed);
            tracing::debug!(
                owner = ?info.signer,
                "prepare to fill auto-stake for lender to: {}",
                deposit
            );
            deposit
        } else {
            tracing::debug!(
                owner = ?info.signer,
                "prepare to fill tickets for lender to: {}",
                info.ticket_account
            );
            info.ticket_account
        };
        UserFillAccounts::Signer(SignerFillAccount(fill))
    }

    fn pop_events(&mut self, num: usize) -> Result<(), EventConsumerError> {
        EventQueue::<CallbackInfo>::from_buffer(&mut self.queue, AccountTag::EventQueue)
            .map(|mut q| q.pop_n(num as u64))
            .map_err(|_| EventConsumerError::InvalidEventQueue(self.market.event_queue))
    }

    fn pending_events(&self) -> Result<usize, EventConsumerError> {
        Ok(OwnedEventQueue::from(self.queue.clone())
            .inner()
            .map_err(|_| EventConsumerError::InvalidEventQueue(self.market.event_queue))?
            .len() as usize)
    }
}

fn make_seed() -> Vec<u8> {
    use rand::RngCore;

    let rng = &mut rand::rngs::OsRng::default();
    let bytes = &mut [0u8; 16];
    rng.fill_bytes(bytes);
    bytes.to_vec()
}

#[derive(Clone, Copy)]
enum EventAccounts {
    Fill(FillAccounts),
    Out(OutAccounts),
}

impl From<&EventAccounts> for Vec<Pubkey> {
    fn from(accounts: &EventAccounts) -> Vec<Pubkey> {
        match accounts {
            EventAccounts::Fill(fill) => fill.into(),
            EventAccounts::Out(out) => out.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FillAccounts {
    user_accounts: UserFillAccounts,
    maker_queue: Option<Pubkey>,
    taker_queue: Option<Pubkey>,
}

#[derive(Debug, Clone, Copy)]
enum UserFillAccounts {
    Margin(MarginFillAccounts),
    Signer(SignerFillAccount),
}

#[derive(Debug, Clone, Copy)]
struct MarginFillAccounts {
    margin_user: Pubkey,
    term_account: Option<Pubkey>,
}

#[derive(Debug, Clone, Copy)]
struct SignerFillAccount(Pubkey);

impl From<&FillAccounts> for Vec<Pubkey> {
    fn from(fill: &FillAccounts) -> Vec<Pubkey> {
        let mut keys = vec![];

        if let Some(queue) = fill.maker_queue {
            keys.push(queue);
        }
        if let Some(queue) = fill.taker_queue {
            keys.push(queue);
        }

        match fill.user_accounts {
            UserFillAccounts::Margin(accs) => {
                keys.push(accs.margin_user);
                if let Some(acc) = accs.term_account {
                    keys.push(acc);
                }
            }
            UserFillAccounts::Signer(acc) => keys.push(acc.0),
        }

        keys
    }
}

#[derive(Debug, Clone, Copy)]
struct OutAccounts {
    out_account: Pubkey,
    user_queue: Option<Pubkey>,
}

impl From<&OutAccounts> for Vec<Pubkey> {
    fn from(out: &OutAccounts) -> Vec<Pubkey> {
        let mut accounts = vec![];

        if let Some(queue) = out.user_queue {
            accounts.push(queue);
        }
        accounts.push(out.out_account);

        accounts
    }
}

macro_rules! maybe_adapter {
    ($it:expr) => {
        if $it.adapter_account_key == Pubkey::default() {
            None
        } else {
            Some($it.adapter_account_key)
        }
    };
}
use maybe_adapter;

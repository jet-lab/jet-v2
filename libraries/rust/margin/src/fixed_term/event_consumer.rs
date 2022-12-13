use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use agnostic_orderbook::state::{
    event_queue::{EventQueue, EventRef, FillEventRef, OutEventRef},
    AccountTag,
};
use anchor_lang::AccountDeserialize;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, packet::PACKET_DATA_SIZE, pubkey::Pubkey,
    signer::Signer, transaction::Transaction,
};
use thiserror::Error;

use jet_fixed_term::{
    control::state::Market,
    margin::state::MarginUser,
    orderbook::state::{CallbackFlags, CallbackInfo},
};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use tracing::instrument;

use super::FixedTermIxBuilder;

const MAX_EVENTS_PER_TX: usize = 32;

#[derive(Error, Debug)]
pub enum EventConsumerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] anyhow::Error),

    #[error("the address is not a market account: {0}")]
    InvalidMarketAccount(Pubkey),

    #[error("the event queue is not readable: {0}")]
    InvalidEventQueue(Pubkey),
}

/// Utility for running consume-events for fixed term markets
pub struct EventConsumer {
    rpc: Arc<dyn SolanaRpcClient>,
    markets: Mutex<HashMap<Pubkey, Arc<Mutex<MarketState>>>>,
}

impl EventConsumer {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            rpc,
            markets: Mutex::new(HashMap::new()),
        }
    }

    /// Load fixed term markets to have their events consumed
    pub async fn load_markets(&self, addresses: &[Pubkey]) -> Result<(), EventConsumerError> {
        let markets = self.rpc.get_multiple_accounts(addresses).await?;

        for (address, market) in addresses.iter().zip(markets) {
            if let Some(data) = market.map(|m| m.data) {
                let structure = Market::try_deserialize(&mut &data[..])
                    .map_err(|_| EventConsumerError::InvalidMarketAccount(*address))?;

                self.markets.lock().unwrap().insert(
                    *address,
                    Arc::new(Mutex::new(MarketState {
                        market_address: *address,
                        market: structure,
                        queue: Vec::new(),
                        users: HashMap::new(),
                        builder: FixedTermIxBuilder::from(structure)
                            .with_payer(&self.rpc.payer().pubkey())
                            .with_crank(&self.rpc.payer().pubkey()),
                    })),
                );
            } else {
                tracing::warn!("missing market {address}");
            }
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
                Some(8 + std::mem::size_of::<MarginUser>()),
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

            if let Some(state) = self.markets.lock().unwrap().get_mut(&structure.market) {
                let mut state = state.lock().unwrap();

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
            for (market, state) in self.markets.lock().unwrap().iter() {
                let state = state.lock().unwrap();

                tracing::trace!(?market, "sync {} total users", state.users.len());
            }
        }

        Ok(())
    }

    /// Sync the event queues
    pub async fn sync_queues(&self) -> Result<(), EventConsumerError> {
        tracing::trace!("sync event queues");

        let (markets, addresses): (Vec<_>, Vec<_>) = self
            .markets
            .lock()
            .unwrap()
            .iter()
            .map(|(addr, state)| {
                let state = state.lock().unwrap();
                (*addr, state.market.event_queue)
            })
            .unzip();
        let accounts = self.rpc.get_multiple_accounts(&addresses).await?;

        for (market, account) in markets.into_iter().zip(accounts) {
            let mut map = self.markets.lock().unwrap();
            let mut market_state = map.get_mut(&market).unwrap().lock().unwrap();

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
                let mut state = state.lock().unwrap();

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
    pub fn pending_events(&self, market: &Pubkey) -> Result<u64, EventConsumerError> {
        let map = self.markets.lock().unwrap();
        let state = match map.get(market) {
            Some(state) => state.lock().unwrap(),
            None => return Ok(0),
        };

        let mut queue_buf = state.queue.clone();
        let queue = EventQueue::<CallbackInfo>::from_buffer(&mut queue_buf, AccountTag::EventQueue)
            .map_err(|_| EventConsumerError::InvalidEventQueue(state.market.event_queue))?;

        Ok(queue.len())
    }
}

#[derive(Clone)]
struct MarketState {
    market_address: Pubkey,
    market: Market,
    queue: Vec<u8>,
    users: HashMap<Pubkey, MarginUser>,
    builder: FixedTermIxBuilder,
}

impl MarketState {
    #[instrument(skip(self, rpc), fields(market = %self.market_address))]
    async fn consume_next(&mut self, rpc: &dyn SolanaRpcClient) -> Result<(), EventConsumerError> {
        let mut queue =
            EventQueue::<CallbackInfo>::from_buffer(&mut self.queue, AccountTag::EventQueue)
                .map_err(|_| EventConsumerError::InvalidEventQueue(self.market.event_queue))?;

        let seed = make_seed();
        let payer = rpc.payer().pubkey();
        let payer_key = rpc.payer();
        let recent_blockhash = rpc.get_latest_blockhash().await?;
        let mut consume_params = vec![];
        let mut consume_tx = Transaction::default();

        for event in queue.iter() {
            match event {
                EventRef::Out(OutEventRef { callback_info, .. }) => {
                    consume_params.push(EventAccounts::Out(OutAccounts {
                        out_account: callback_info.out_account,
                        user_queue: callback_info.adapter(),
                    }))
                }

                EventRef::Fill(FillEventRef {
                    maker_callback_info,
                    taker_callback_info,
                    ..
                }) => {
                    let fill_account = maker_callback_info.fill_account;
                    let mut loan_account = None;

                    if maker_callback_info
                        .flags
                        .contains(CallbackFlags::AUTO_STAKE)
                    {
                        // If auto-stake is enabled for lending, then consuming the event
                        // requires passing in the right address for the `TermDeposit` account
                        // to be created now that the loan has been filled

                        if let Some(maker_user) = self.users.get_mut(&fill_account) {
                            // In this case, the maker is using a margin account, so we derive
                            // the deposit account based on a sequence number in the account state
                            let seed = maker_user.assets.next_deposit_seqno.to_le_bytes();
                            maker_user.assets.next_deposit_seqno += 1;

                            loan_account =
                                Some(self.builder.term_deposit_key(&fill_account, &seed));
                        } else {
                            // In this case the maker doesn't have a margin account, so we derive
                            // the deposit account based on the random seed for this transaction
                            loan_account =
                                Some(self.builder.term_deposit_key(&fill_account, &seed));
                        }

                        tracing::debug!(
                            owner = ?maker_callback_info.owner,
                            "prepare to fill auto-stake for lender to: {}",
                            loan_account.as_ref().unwrap()
                        );
                    } else if maker_callback_info.flags.contains(CallbackFlags::NEW_DEBT) {
                        // If this fill is issuing debt, then consuming requires passing in
                        // the address for the `TermLoan` account to be created for tracking
                        // the user debt

                        if let Some(maker_user) = self.users.get_mut(&fill_account) {
                            // In this case, the maker is using a margin account, so we
                            // derive the new `TermLoan` account based on the debt sequence
                            // number in the account state
                            let seed = maker_user.debt.next_new_term_loan_seqno.to_le_bytes();
                            maker_user.debt.next_new_term_loan_seqno += 1;

                            loan_account = Some(self.builder.term_loan_key(&fill_account, &seed));

                            tracing::debug!(
                                owner = ?maker_callback_info.owner,
                                "prepare to fill debt for borrower to: {}",
                                loan_account.as_ref().unwrap()
                            );
                        } else {
                            tracing::error!(
                                "unexpected debt fill with non-margin account: {}",
                                maker_callback_info.fill_account
                            );
                        }
                    }

                    consume_params.push(EventAccounts::Fill(FillAccounts {
                        fill_account: maker_callback_info.fill_account,
                        maker_queue: maker_callback_info.adapter(),
                        taker_queue: taker_callback_info.adapter(),
                        deposit_account: loan_account,
                    }))
                }
            }

            let consume_ix = [
                ComputeBudgetInstruction::set_compute_unit_limit(800_000),
                self.builder.consume_events(&seed, &consume_params).unwrap(),
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

        queue.pop_n(consume_params.len() as u64);
        rpc.send_and_confirm_transaction(&consume_tx).await?;
        Ok(())
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

#[derive(Clone, Copy)]
struct FillAccounts {
    fill_account: Pubkey,
    deposit_account: Option<Pubkey>,
    maker_queue: Option<Pubkey>,
    taker_queue: Option<Pubkey>,
}

impl From<&FillAccounts> for Vec<Pubkey> {
    fn from(fill: &FillAccounts) -> Vec<Pubkey> {
        let mut accounts = vec![fill.fill_account];

        accounts.extend(fill.deposit_account);
        accounts.extend(fill.maker_queue);
        accounts.extend(fill.taker_queue);

        accounts
    }
}

#[derive(Clone, Copy)]
struct OutAccounts {
    out_account: Pubkey,
    user_queue: Option<Pubkey>,
}

impl From<&OutAccounts> for Vec<Pubkey> {
    fn from(out: &OutAccounts) -> Vec<Pubkey> {
        let mut accounts = vec![out.out_account];

        accounts.extend(out.user_queue);
        accounts
    }
}

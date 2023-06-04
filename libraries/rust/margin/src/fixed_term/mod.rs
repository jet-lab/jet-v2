#![allow(missing_docs)]

pub mod auto_roll_servicer;
pub mod error;
pub mod event_consumer;
mod ix_builder;
pub mod settler;

use futures::future::{join_all, try_join_all};
pub use ix_builder::*;

use anchor_lang::AccountDeserialize;
use jet_simulation::SolanaRpcClient;
use jet_solana_client::rpc::AccountFilter;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::fixed_term::settler::settler;
use crate::util::no_dupe_queue::AsyncNoDupeQueue;

use self::{
    auto_roll_servicer::AutoRollServicer,
    event_consumer::{download_markets, EventConsumer},
    settler::Settler,
};

/// Find all the fixed term markets.
/// TODO: generalize this to rpc client, code from liquidator may be useful
pub async fn find_markets(
    rpc: &Arc<dyn SolanaRpcClient>,
) -> anyhow::Result<HashMap<Pubkey, Market>> {
    Ok(rpc
        .get_program_accounts(
            &jet_fixed_term::ID,
            vec![AccountFilter::DataSize(std::mem::size_of::<Market>() + 8)],
        )
        .await?
        .into_iter()
        .flat_map(|(k, account)| {
            match Market::try_deserialize(&mut &account.data[..]).map_err(anyhow::Error::from) {
                Ok(mkt) => Some((k, mkt)),
                Err(_) => None,
            }
        })
        .collect())
}

pub struct Crank {
    pub consumer: EventConsumer,
    pub settlers: Vec<Settler>,
    pub servicers: Vec<AutoRollServicer>,
    pub market_addrs: Vec<Pubkey>,
    pub consumer_delay: Duration,
}

impl Crank {
    pub async fn new(
        rpc: Arc<dyn SolanaRpcClient>,
        market_addrs: &[Pubkey],
    ) -> anyhow::Result<Self> {
        let markets = download_markets(rpc.as_ref(), market_addrs).await?;
        let consumer = EventConsumer::new(rpc.clone());
        let mut settlers = vec![];
        let mut servicers = vec![];
        for market in markets {
            let margin_accounts = AsyncNoDupeQueue::new();
            let ix = FixedTermIxBuilder::new_from_state(rpc.payer().pubkey(), &market);
            consumer.insert_market(market, Some(margin_accounts.clone()));
            let settler = settler(rpc.clone(), ix.clone(), margin_accounts, Default::default())?;
            settlers.push(settler);
            servicers.push(AutoRollServicer::new(rpc.clone(), ix))
        }

        Ok(Self {
            consumer,
            settlers,
            servicers,
            market_addrs: market_addrs.to_vec(),
            consumer_delay: Duration::from_secs(2),
        })
    }

    /// Consumes all events that are currently in the queue, then settles any
    /// accounts that need to be settled due to those events, then returns.
    pub async fn run_once(&self) -> anyhow::Result<()> {
        self.consumer
            .sync_and_consume_all(&self.market_addrs)
            .await?;
        try_join_all(self.settlers.iter().map(|s| s.process_all())).await?;
        join_all(self.servicers.iter().map(|s| s.service_all())).await;
        Ok(())
    }

    /// Continuously consumes any events and settles any accounts as they need
    /// to be processed.
    pub async fn run_forever(self) {
        let mut jobs = vec![];
        for settler in self.settlers {
            jobs.push(tokio::spawn(async move { settler.process_forever().await }));
        }
        for servicer in self.servicers {
            jobs.push(tokio::spawn(
                async move { servicer.service_forever().await },
            ));
        }
        self.consumer
            .sync_and_consume_forever(&self.market_addrs, self.consumer_delay)
            .await;
        try_join_all(jobs).await.unwrap();
    }
}

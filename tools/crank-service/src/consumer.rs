use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_margin_sdk::fixed_term::{FixedTermIxBuilder, Market, OwnedEventQueue};
use log::{error, trace};
use solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer};
use tokio::task::JoinHandle;

use crate::client::Client;

pub struct Consumer {
    client: Client,
    ix: FixedTermIxBuilder,
}

/// Convenient struct for logging successful event consumption
#[allow(dead_code)]
#[derive(Debug)]
struct EventConsumptionData {
    pub market: Pubkey,
    pub num_consumed: u32,
    pub signature: Signature,
}

/// Convenient struct for logging event consumption errors
#[allow(dead_code)]
#[derive(Debug)]
struct EventConsumptionErrorData<E: Into<anyhow::Error>> {
    pub market: Pubkey,
    pub error: E,
}

impl Consumer {
    pub fn spawn(client: Client, market: Pubkey) -> Result<JoinHandle<Result<()>>> {
        Ok(tokio::spawn(async move {
            Self::init(client, market).await?.run().await
        }))
    }

    async fn init(client: Client, market: Pubkey) -> Result<Self> {
        let manager = {
            let data = client.conn.get_account_data(&market).await.map_err(|e| {
                error!("failed to fetch data for market [{market}]. Error: {e}");
                e
            })?;
            Market::try_deserialize(&mut data.as_slice())?
        };
        let ix = FixedTermIxBuilder::from(manager)
            .with_crank(&client.signer.pubkey())
            .with_payer(&client.signer.pubkey());

        trace!("consumer initialized for market: [{}]", market);
        Ok(Self { client, ix })
    }

    async fn run(self) -> Result<()> {
        let market = self.ix.market();
        loop {
            let mut queue = self.fetch_queue().await?;
            trace!("fetched queue for market: [{}]", market);
            if queue.is_empty()? {
                trace!("empty queue for market: [{}]", market);
                // nothing to consume
                continue;
            }
            let params = queue.consume_events_params()?;
            let consume_ix = self.ix.consume_events(&params)?;

            // TODO: metrics and error handling
            match self.client.sign_send_ix(consume_ix).await {
                Ok(s) => {
                    trace!(
                        "event consumption success: [{:?}]",
                        EventConsumptionData {
                            market,
                            num_consumed: params.num_events,
                            signature: s,
                        }
                    );
                }
                Err(e) => {
                    error!(
                        "failed to consume events: [{:?}]",
                        EventConsumptionErrorData { market, error: e }
                    );
                    continue;
                }
            }
        }
    }

    async fn fetch_queue<'a>(&self) -> Result<OwnedEventQueue> {
        let data = self
            .client
            .conn
            .get_account_data(&self.ix.event_queue())
            .await
            .map_err(|e| {
                error!(
                    "failed to fetch queue for market [{}]. Error: {e}",
                    self.ix.market()
                );
                e
            })?;

        Ok(OwnedEventQueue::from(data))
    }
}

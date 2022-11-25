use agnostic_orderbook::state::{event_queue::{EventQueue}, AccountTag};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_margin_sdk::{bonds::{BondManager, BondsIxBuilder}, jet_bonds::orderbook::state::CallbackInfo};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::client::Client;

pub struct Consumer {
    client: Client,
    ix: BondsIxBuilder,
}

impl Consumer {
    pub async fn spawn(client: Client, market: Pubkey) -> Result<()> {
        let manager = { let data = client.conn.get_account_data(&market).await?; 
        BondManager::try_deserialize(&mut data.as_slice())?
        };
        let ix = BondsIxBuilder::from(manager).with_crank(&client.signer.pubkey())
        .with_payer(&client.signer.pubkey());

        Self {
            client,
            ix
        }.run().await
    }

    async fn run(self) -> Result<()> {
        loop {
            let mut queue = self.fetch_queue().await?;
            if queue.is_empty()? {
                // nothing to consume
                continue;
            }

            let consume_ix = self.ix.consume_events(queue.inner()?)?;

            // TODO: metrics and error handling
            match self.client.sign_send_ix(consume_ix).await {
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
        Ok(())
    }

    async fn fetch_queue<'a>(&self) -> Result<OwnedQueue> {
        let data = self.client.conn.get_account_data(&self.ix.event_queue()).await?;

        Ok(OwnedQueue::from(data))
    }
}

struct OwnedQueue(Vec<u8>);

impl OwnedQueue {
    pub fn inner(&mut self) -> Result<EventQueue<CallbackInfo>> {
        EventQueue::from_buffer(&mut self.0, AccountTag::EventQueue)
        .map_err(anyhow::Error::from)
    }

    pub fn is_empty(&mut self) -> Result<bool> {
        Ok(self.inner()?.iter().next().is_none())
    }
}

impl<T: Into<Vec<u8>>> From<T> for OwnedQueue {
    fn from(data: T) -> Self {
        Self(data.into())
    }
}
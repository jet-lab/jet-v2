use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_margin_sdk::bonds::{BondManager, BondsIxBuilder, OwnedEventQueue};
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use tokio::task::JoinHandle;

use crate::client::Client;

pub struct Consumer {
    client: Client,
    ix: BondsIxBuilder,
    is_verbose: bool,
}

impl Consumer {
    pub fn spawn(
        client: Client,
        market: Pubkey,
        is_verbose: bool,
    ) -> Result<JoinHandle<Result<()>>> {
        Ok(tokio::spawn(async move {
            Self::init(client, market, is_verbose).await?.run().await
        }))
    }

    async fn init(client: Client, market: Pubkey, is_verbose: bool) -> Result<Self> {
        let manager = {
            let data = client.conn.get_account_data(&market).await?;
            BondManager::try_deserialize(&mut data.as_slice())?
        };
        let ix = BondsIxBuilder::from(manager)
            .with_crank(&client.signer.pubkey())
            .with_payer(&client.signer.pubkey());

        Ok(Self {
            client,
            ix,
            is_verbose,
        })
    }

    async fn run(self) -> Result<()> {
        loop {
            let mut queue = self.fetch_queue().await?;
            if queue.is_empty()? {
                // nothing to consume
                continue;
            }
            let params = queue.consume_events_params()?;
            let consume_ix = self.ix.consume_events(&params)?;

            // TODO: metrics and error handling
            match self.client.sign_send_ix(consume_ix).await {
                Ok(s) => {
                    if self.is_verbose {
                        println!(
                            "Success! Market Key: [{}] Events consumed: [{}] Signature: [{}]",
                            self.ix.manager(),
                            params.num_events,
                            s
                        )
                    }
                }
                Err(_) => continue,
            }
        }
    }

    async fn fetch_queue<'a>(&self) -> Result<OwnedEventQueue> {
        let data = self
            .client
            .conn
            .get_account_data(&self.ix.event_queue())
            .await?;

        Ok(OwnedEventQueue::from(data))
    }
}
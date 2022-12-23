use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use jet_instructions::test_service;

use crate::{
    client::{ClientResult, ClientState},
    UserNetworkInterface,
};

/// Client for interacting with the test-service program
#[derive(Clone)]
pub struct TestServiceClient<I> {
    client: Arc<ClientState<I>>,
}

impl<I: UserNetworkInterface> TestServiceClient<I> {
    pub(crate) fn new(client: Arc<ClientState<I>>) -> Self {
        Self { client }
    }

    /// Request a number of tokens be minted and deposited in the current user wallet
    pub async fn token_request(&self, mint: &Pubkey, amount: u64) -> ClientResult<I, ()> {
        let mut ixns = vec![];
        let destination = self.client.with_wallet_account(mint, &mut ixns).await?;

        ixns.push(test_service::token_request(
            &self.client.signer(),
            mint,
            &destination,
            amount,
        ));

        self.client.send(&ixns).await
    }
}

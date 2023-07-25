use std::sync::Arc;

use crate::client::ClientState;


pub struct GovernanceClient {
    client: Arc<ClientState>,
}

impl GovernanceClient {
    pub fn new(client: Arc<ClientState>) -> Self {
        Self { client }
    }
}
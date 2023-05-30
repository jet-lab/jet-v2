use std::str::FromStr;

use serde::{Deserialize, Serialize};
use solana_sdk::hash::Hash;

use crate::rpc::{ClientError, SolanaRpc};

const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

/// Description for the Solana network a client may connect to
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum NetworkKind {
    /// The public mainnet-beta network
    Mainnet,

    /// The public network for development testing
    Devnet,

    /// A non-public network for testing
    Localnet,
}

impl NetworkKind {
    /// Determine the network type based on its genesis hash
    pub fn from_genesis_hash(network_genesis_hash: &Hash) -> Self {
        if *network_genesis_hash == Hash::from_str(MAINNET_HASH).unwrap() {
            NetworkKind::Mainnet
        } else if *network_genesis_hash == Hash::from_str(DEVNET_HASH).unwrap() {
            NetworkKind::Devnet
        } else {
            NetworkKind::Localnet
        }
    }

    /// Determine the network type for a given interface
    pub async fn from_interface(network: &dyn SolanaRpc) -> Result<Self, ClientError> {
        let network_hash = network.get_genesis_hash().await?;
        Ok(Self::from_genesis_hash(&network_hash))
    }
}

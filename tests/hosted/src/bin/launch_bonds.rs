use std::sync::Arc;

use anyhow::Result;
use hosted_tests::{bonds::TestManager, margin::MarginClient};
use jet_simulation::solana_rpc_api::RpcConnection;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let rpc = Arc::new(RpcConnection::new_local_funded()?);
    let margin = MarginClient::new(rpc.clone());
    margin.create_authority_if_missing().await?;
    margin
        .register_adapter_if_unregistered(&jet_bonds::ID)
        .await?;

    TestManager::new(rpc).await?.with_bonds().await?;

    Ok(())
}

use std::sync::Arc;

use anyhow::Result;
use hosted_tests::bonds::TestManager;
use jet_simulation::solana_rpc_api::RpcConnection;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let rpc = RpcConnection::new_local_funded()?;

    let manager = TestManager::full(Arc::new(rpc)).await?;

    Ok(())
}

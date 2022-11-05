use std::sync::Arc;
use tokio::sync::OnceCell;

use jet_simulation::runtime::TestRuntime;
use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};

static RUNTIME: OnceCell<Arc<dyn SolanaRpcClient>> = OnceCell::const_new();

pub async fn runtime() -> Arc<dyn SolanaRpcClient> {
    RUNTIME.get_or_init(build_test_runtime).await.clone()
}

/// puts the bare minimum code into conditional compilation, so the compiler
/// will always check the validity of as much code as possible.
async fn build_test_runtime() -> Arc<dyn SolanaRpcClient> {
    #[cfg(feature = "localnet")]
    {
        localnet_runtime().await
    }
    #[cfg(not(feature = "localnet"))]
    {
        simulation_runtime()
    }
}

#[allow(unused)]
fn simulation_runtime() -> Arc<dyn SolanaRpcClient> {
    let runtime = jet_simulation::create_test_runtime![
        jet_test_service,
        jet_bonds,
        jet_control,
        jet_margin,
        jet_metadata,
        jet_airspace,
        jet_margin_pool,
        jet_margin_swap,
        (
            orca_swap_v1::id(),
            orca_swap_v1::processor::Processor::process
        ),
        (
            orca_swap_v2::id(),
            orca_swap_v2::processor::Processor::process
        ),
        (
            spl_token_swap_v2::id(),
            spl_token_swap_v2::processor::Processor::process
        ),
        (
            spl_associated_token_account::ID,
            spl_associated_token_account::processor::process_instruction
        )
    ];

    Arc::new(runtime)
}

#[allow(unused)]
async fn localnet_runtime() -> Arc<dyn SolanaRpcClient> {
    let runtime = RpcConnection::new_local_funded().await.unwrap();

    Arc::new(runtime)
}

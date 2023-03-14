use anchor_lang::prelude::{AccountInfo, Pubkey};
use solana_sdk::entrypoint::ProgramResult;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;

use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};

pub use jet_simulation::{DeterministicKeygen, Keygen, RandomKeygen};

/// If you don't provide a name, gets the name of the current function name and
/// uses it to create a test context. Only use this way when called directly in
/// the test function. If you want to call this in a helper function, pass a
/// name that is unique to the individual test.
#[macro_export]
macro_rules! solana_test_context {
    () => {
        $crate::runtime::SolanaTestContext::new(&$crate::fn_name_and_try_num!()).await
    };
    ($name:expr) => {
        $crate::runtime::SolanaTestContext::new($name).await
    };
}

/// Returns a string with the fully qualified name of the current function,
/// followed by the nextest attempt number (increments on retry).  
/// Example: "liquidate::can_withdraw_some_during_liquidation-try_1"
#[macro_export]
macro_rules! fn_name_and_try_num {
    () => {
        format!(
            "{}-try_{}",
            $crate::runtime::__type_name_of(|| {}).replace("::{{closure}}", ""),
            $crate::runtime::current_test_attempt_number()
        )
    };
}

pub fn __type_name_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

pub fn current_test_attempt_number() -> String {
    std::env::var("__NEXTEST_ATTEMPT").unwrap_or("1".to_string())
}

#[derive(Clone)]
pub struct SolanaTestContext {
    pub rpc: Arc<dyn SolanaRpcClient>,
    pub keygen: Arc<dyn Keygen>,
}

impl SolanaTestContext {
    pub async fn new(test_name: &str) -> SolanaTestContext {
        let keygen = Arc::new(DeterministicKeygen::new(test_name));
        let rpc = init_runtime(keygen.generate_key()).await;

        rpc.airdrop(&rpc.payer().pubkey(), 10_000 * LAMPORTS_PER_SOL)
            .await
            .unwrap();

        Self { rpc, keygen }
    }

    pub fn generate_key(&self) -> Keypair {
        self.keygen.generate_key()
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair, anyhow::Error> {
        let wallet = self.generate_key();
        self.rpc
            .airdrop(&wallet.pubkey(), sol_amount * LAMPORTS_PER_SOL)
            .await?;

        Ok(wallet)
    }
}

async fn init_runtime(payer: Keypair) -> Arc<dyn SolanaRpcClient> {
    if cfg!(feature = "localnet") {
        localnet_runtime(payer).await
    } else {
        simulation_runtime(payer).await
    }
}

async fn localnet_runtime(payer: Keypair) -> Arc<dyn SolanaRpcClient> {
    Arc::new(RpcConnection::new_optimistic(
        payer,
        "http://127.0.0.1:8899",
    ))
}

async fn simulation_runtime(payer: Keypair) -> Arc<dyn SolanaRpcClient> {
    let _ = env_logger::builder().is_test(false).try_init();
    let runtime = jet_simulation::create_test_runtime![
        jet_test_service,
        jet_fixed_term,
        jet_control,
        jet_margin,
        jet_metadata,
        jet_airspace,
        jet_margin_pool,
        jet_margin_swap,
        (spl_token::ID, spl_token::processor::Processor::process),
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
        ),
        (
            saber_program::id(),
            saber_program::processor::Processor::process
        ),
        (anchor_spl::dex::id(), openbook_processor),
    ];

    Arc::new(runtime.rpc(payer))
}

// Register OpenBook, converting a DexError to ProgramError
fn openbook_processor(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    anchor_spl::dex::serum_dex::state::State::process(program_id, accounts, input)
        .map_err(|e| e.into())
}

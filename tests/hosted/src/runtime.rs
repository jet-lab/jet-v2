use rand::rngs::mock::StepRng;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::sync::OnceCell;

use jet_simulation::runtime::TestRuntime;
use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};

/// If you don't provide a name, gets the name of the current function name and
/// uses it to create a test context. Only use this way when called directly in
/// the test function. If you want to call this in a helper function, pass a
/// name that is unique to the individual test.
#[macro_export]
macro_rules! solana_test_context {
    () => {
        $crate::runtime::SolanaTestContext::new($crate::fn_name!()).await
    };
    ($name:expr) => {
        $crate::runtime::SolanaTestContext::new($name).await
    };
}

/// Generates a string that is unique to the containing function.
#[macro_export]
macro_rules! fn_name {
    () => {
        $crate::runtime::__type_name_of(|| {})
    };
}
pub fn __type_name_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

#[derive(Clone)]
pub struct SolanaTestContext {
    pub rpc: Arc<dyn SolanaRpcClient>,
    pub keygen: Arc<dyn Keygen>,
}

impl SolanaTestContext {
    pub async fn new(test_name: &str) -> SolanaTestContext {
        SolanaTestContext {
            rpc: get_runtime().await,
            keygen: if cfg!(feature = "localnet") {
                Arc::new(RandomKeygen) // so retries will work
            } else {
                Arc::new(DeterministicKeygen::new(test_name))
            },
        }
    }

    pub fn generate_key(&self) -> Keypair {
        self.keygen.generate_key()
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair, anyhow::Error> {
        jet_simulation::create_wallet(&self.rpc, sol_amount * LAMPORTS_PER_SOL).await
    }
}

pub trait Keygen: Send + Sync {
    fn generate_key(&self) -> Keypair;
}

#[derive(Clone)]
pub struct DeterministicKeygen(Arc<Mutex<RefCell<MockRng>>>);
impl DeterministicKeygen {
    pub fn new(seed: &str) -> Self {
        let seed: u64 = seed
            .as_bytes()
            .chunks(8)
            .map(|chunk| {
                let mut a = [0u8; 8];
                a[..chunk.len()].copy_from_slice(chunk);
                u64::from_le_bytes(a)
            })
            .fold(0, |acc, next| acc.wrapping_add(next));
        Self(Arc::new(Mutex::new(RefCell::new(MockRng(StepRng::new(
            seed, 1,
        ))))))
    }
}
impl Keygen for DeterministicKeygen {
    fn generate_key(&self) -> Keypair {
        Keypair::generate(&mut *self.0.lock().unwrap().borrow_mut())
    }
}

#[derive(Clone)]
pub struct RandomKeygen;
impl Keygen for RandomKeygen {
    fn generate_key(&self) -> Keypair {
        Keypair::new()
    }
}

struct MockRng(StepRng);
impl rand::CryptoRng for MockRng {}
impl rand::RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}

async fn get_runtime() -> Arc<dyn SolanaRpcClient> {
    if cfg!(feature = "localnet") {
        localnet_runtime().await
    } else {
        simulation_runtime().await
    }
}

async fn localnet_runtime() -> Arc<dyn SolanaRpcClient> {
    Arc::new(RpcConnection::new_local_funded().await.unwrap())
}

async fn simulation_runtime() -> Arc<dyn SolanaRpcClient> {
    SIMULATION
        .get_or_init(build_simulation_runtime)
        .await
        .clone()
}

static SIMULATION: OnceCell<Arc<dyn SolanaRpcClient>> = OnceCell::const_new();

async fn build_simulation_runtime() -> Arc<dyn SolanaRpcClient> {
    let runtime = jet_simulation::create_test_runtime![
        jet_test_service,
        jet_market,
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

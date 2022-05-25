use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use anyhow::Error;
use rand::rngs::mock::StepRng;
use tokio::sync::OnceCell;

use jet_simulation::margin::MarginClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::MarginPoolConfig;
use jet_metadata::TokenKind;

use jet_simulation::tokens::TokenManager;
use jet_simulation::TestRuntime;
use jet_solana_rpc_api::SolanaRpcClient;

static TEST_CONTEXT: OnceCell<MarginTestContext> = OnceCell::const_new();

pub async fn test_context() -> &'static MarginTestContext {
    TEST_CONTEXT
        .get_or_init(|| async {
            let ctx = MarginTestContext::new().await.unwrap();

            ctx
        })
        .await
}

pub struct MarginPoolSetupInfo {
    pub token: Pubkey,
    pub fee_vault: Pubkey,
    pub kind: TokenKind,
    pub weight: u16,
    pub config: MarginPoolConfig,
}

/// Utilities for testing things in the context of the margin system
pub struct MarginTestContext {
    pub rpc: Arc<dyn SolanaRpcClient>,
    pub tokens: TokenManager,
    pub margin: MarginClient,

    pub authority: Keypair,
    pub payer: Keypair,

    rng: Mutex<RefCell<MockRng>>,
}

impl MarginTestContext {
    pub async fn new() -> Result<Self, Error> {
        let runtime = Arc::new(jet_simulation::create_test_runtime![
            jet_control,
            jet_margin,
            jet_metadata,
            jet_margin_pool,
            jet_margin_swap
        ]);
        let payer = Keypair::from_bytes(&runtime.payer().to_bytes()).unwrap();
        let rng = MockRng(StepRng::new(0, 1));
        let ctx = MarginTestContext {
            tokens: TokenManager::new(runtime.clone()),
            margin: MarginClient::new(runtime.clone()),
            authority: Keypair::new(),
            rng: Mutex::new(RefCell::new(rng)),

            rpc: runtime,
            payer,
        };

        ctx.margin.create_authority().await?;
        ctx.margin.register_adapter(&jet_margin_pool::ID).await?;
        ctx.margin.register_adapter(&jet_margin_swap::ID).await?;

        Ok(ctx)
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair, Error> {
        jet_simulation::create_wallet(&self.rpc, sol_amount * LAMPORTS_PER_SOL).await
    }

    pub fn generate_key(&self) -> Keypair {
        Keypair::generate(&mut *self.rng.lock().unwrap().borrow_mut())
    }

    /// Generate a new wallet keypair for a liquidator with the pubkey that
    /// stores the [LiquidatorMetadata]
    pub async fn create_liquidator(&self, sol_amount: u64) -> Result<Keypair, Error> {
        let liquidator = self.create_wallet(sol_amount).await?;

        self.margin
            .set_liquidator_metadata(liquidator.pubkey(), true)
            .await?;
        Ok(liquidator)
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

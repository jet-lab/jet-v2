use std::sync::Arc;

use anyhow::Error;
use jet_margin_sdk::solana::transaction::InverseSendTransactionBuilder;
use jet_margin_sdk::util::data::With;

use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_sdk::test_service::{minimal_environment, MarginPoolConfig};
use jet_metadata::TokenKind;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::runtime::SolanaTestContext;
use crate::{margin::MarginClient, tokens::TokenManager};

/// If you don't provide a name, gets the name of the current function name and
/// uses it to create a test context. Only use this way when called directly in
/// the test function. If you want to call this in a helper function, pass a
/// name that is unique to the individual test.
#[macro_export]
macro_rules! margin_test_context {
    () => {
        $crate::margin_test_context!($crate::fn_name!())
    };
    ($name:expr) => {
        std::sync::Arc::new(
            $crate::context::MarginTestContext::new($name)
                .await
                .unwrap(),
        )
    };
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
    pub solana: SolanaTestContext,
}

impl std::fmt::Debug for MarginTestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarginTestContext")
            .field("authority", &self.authority)
            .field("payer", &self.payer)
            .finish()
    }
}

impl From<SolanaTestContext> for MarginTestContext {
    fn from(solana: SolanaTestContext) -> Self {
        let payer = Keypair::from_bytes(&solana.rpc.payer().to_bytes()).unwrap();
        MarginTestContext {
            tokens: TokenManager::new(solana.clone()),
            margin: MarginClient::new(solana.rpc.clone(), &payer.pubkey().to_string()[0..8]),
            authority: Keypair::new(),
            rpc: solana.rpc.clone(),
            solana,
            payer,
        }
    }
}

// impl MarginTestContext {
//     #[cfg(not(feature = "localnet"))]
//     pub async fn new() -> Result<Self, Error> {
//         use jet_simulation::runtime::TestRuntime;
//         use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};
//         let runtime = jet_simulation::create_test_runtime![
//             jet_test_service,
//             jet_bonds,
//             jet_control,
//             jet_margin,
//             jet_metadata,
//             jet_airspace,
//             jet_margin_pool,
//             jet_margin_swap,
//             orca_whirlpool_program,
//             (
//                 orca_swap_v1::id(),
//                 orca_swap_v1::processor::Processor::process
//             ),
//             (
//                 orca_swap_v2::id(),
//                 orca_swap_v2::processor::Processor::process
//             ),
//             (
//                 spl_token_swap_v2::id(),
//                 spl_token_swap_v2::processor::Processor::process
//             ),
//             (
//                 spl_associated_token_account::ID,
//                 spl_associated_token_account::processor::process_instruction
//             ),
//             (
//                 saber_client::id(),
//                 saber_program::processor::Processor::process
//             ),
//         ];

//         Self::new_with_runtime(Arc::new(runtime)).await
//     }

//     #[cfg(feature = "localnet")]
//     pub async fn new() -> Result<Self, Error> {
//         use jet_simulation::solana_rpc_api::RpcConnection;

//         let solana_config =
//             solana_cli_config::Config::load(solana_cli_config::CONFIG_FILE.as_ref().unwrap())
//                 .unwrap_or_default();

//         let payer_key_json = std::fs::read_to_string(&solana_config.keypair_path)?;
//         let payer_key_bytes: Vec<u8> = serde_json::from_str(&payer_key_json)?;
//         let payer = Keypair::from_bytes(&payer_key_bytes).unwrap();

//         let rpc = RpcConnection::new_optimistic(
//             Keypair::from_bytes(&payer_key_bytes).unwrap(),
//             "http://127.0.0.1:8899",
//         );

//         let runtime = Arc::new(rpc);

//         let rng = MockRng(StepRng::new(0, 1));
//         let ctx = MarginTestContext {
//             tokens: TokenManager::new(runtime.clone()),
//             margin: MarginClient::new(runtime.clone()),
//             authority: Keypair::new(),
//             rpc: solana.rpc.clone(),
//             solana,
//             payer,
//         }
//     }
impl MarginTestContext {
    pub async fn new(seed: &str) -> Result<Self, Error> {
        Self::new_with_runtime(SolanaTestContext::new(seed).await).await
    }

    pub async fn new_with_runtime(runtime: SolanaTestContext) -> Result<Self, Error> {
        let ctx: Self = runtime.into();

        minimal_environment(ctx.payer.pubkey())
            .with(ctx.margin.create_airspace_ix(false).into())
            .send_and_confirm_condensed(&ctx.rpc)
            .await?;

        Ok(ctx)
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair, Error> {
        jet_simulation::create_wallet(&self.rpc, sol_amount * LAMPORTS_PER_SOL).await
    }

    pub fn generate_key(&self) -> Keypair {
        self.solana.keygen.generate_key()
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

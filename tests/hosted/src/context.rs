use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anchor_lang::prelude::Rent;
use anyhow::Error;
use jet_margin_sdk::jet_margin_pool::PoolFlags;
use jet_margin_sdk::solana::transaction::{condense, SendTransactionBuilder};
use rand::rngs::mock::StepRng;
use tokio::sync::OnceCell;

use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_sdk::test_service::{
    init_environment, AirspaceConfig, AirspaceTokenConfig, BondMarketConfig, EnvironmentConfig,
    MarginPoolConfig, TokenDescription,
};
use jet_metadata::TokenKind;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::{margin::MarginClient, tokens::TokenManager};

static TEST_CONTEXT: OnceCell<MarginTestContext> = OnceCell::const_new();

pub async fn test_context() -> &'static MarginTestContext {
    TEST_CONTEXT
        .get_or_init(|| async { MarginTestContext::new().await.unwrap() })
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

impl std::fmt::Debug for MarginTestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarginTestContext")
            .field("authority", &self.authority)
            .field("payer", &self.payer)
            .finish()
    }
}

impl MarginTestContext {
    #[cfg(not(feature = "localnet"))]
    pub async fn new() -> Result<Self, Error> {
        use jet_simulation::runtime::TestRuntime;
        use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};
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

        Self::new_with_runtime(Arc::new(runtime)).await
    }

    #[cfg(feature = "localnet")]
    pub async fn new() -> Result<Self, Error> {
        use jet_simulation::solana_rpc_api::RpcConnection;

        let solana_config =
            solana_cli_config::Config::load(solana_cli_config::CONFIG_FILE.as_ref().unwrap())
                .unwrap_or_default();

        let payer_key_json = std::fs::read_to_string(&solana_config.keypair_path)?;
        let payer_key_bytes: Vec<u8> = serde_json::from_str(&payer_key_json)?;
        let payer = Keypair::from_bytes(&payer_key_bytes).unwrap();

        let rpc = RpcConnection::new_optimistic(
            Keypair::from_bytes(&payer_key_bytes).unwrap(),
            "http://127.0.0.1:8899",
        );

        let runtime = Arc::new(rpc);

        let rng = MockRng(StepRng::new(0, 1));
        let ctx = MarginTestContext {
            tokens: TokenManager::new(runtime.clone()),
            margin: MarginClient::new(runtime.clone()),
            authority: Keypair::new(),
            rng: Mutex::new(RefCell::new(rng)),
            rpc: runtime,
            payer,
        };

        Ok(ctx)
    }

    pub async fn new_with_runtime(runtime: Arc<dyn SolanaRpcClient>) -> Result<Self, Error> {
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

        let init_txs = init_environment(&ctx.default_config(), &Rent::default())?;
        for tx in condense(&init_txs, &ctx.payer)? {
            ctx.rpc.send_and_confirm(tx).await?;
        }

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

    fn default_config(&self) -> EnvironmentConfig {
        const DEFAULT_POOL_CONFIG: MarginPoolConfig = MarginPoolConfig {
            borrow_rate_0: 10,
            borrow_rate_1: 20,
            borrow_rate_2: 30,
            borrow_rate_3: 40,
            utilization_rate_1: 10,
            utilization_rate_2: 20,
            management_fee_rate: 10,
            flags: PoolFlags::ALLOW_LENDING.bits(),
        };

        EnvironmentConfig {
            authority: self.payer.pubkey(),
            tokens: vec![
                TokenDescription {
                    symbol: "USDC".to_owned(),
                    name: "USDC".to_owned(),
                    decimals: 6,
                    precision: 2,
                },
                TokenDescription {
                    symbol: "BTC".to_owned(),
                    name: "Bitcoin".to_owned(),
                    decimals: 6,
                    precision: 6,
                },
            ],
            airspaces: vec![AirspaceConfig {
                name: "default".to_owned(),
                is_restricted: false,
                tokens: HashMap::from_iter([
                    (
                        "USDC".to_owned(),
                        AirspaceTokenConfig {
                            collateral_weight: 100,
                            max_leverage: 20_00,
                            margin_pool_config: Some(DEFAULT_POOL_CONFIG),
                            bond_markets: vec![BondMarketConfig {
                                duration: 5,
                                min_order_size: 10,
                            }],
                        },
                    ),
                    (
                        "Bitcoin".to_owned(),
                        AirspaceTokenConfig {
                            collateral_weight: 75,
                            max_leverage: 10_00,
                            margin_pool_config: Some(DEFAULT_POOL_CONFIG),
                            bond_markets: vec![],
                        },
                    ),
                ]),
            }],
        }
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

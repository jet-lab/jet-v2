use std::sync::Arc;

use anyhow::Error;
use jet_client::config::{AirspaceInfo, DexInfo, JetAppConfig, TokenInfo};
use jet_client::programs::ORCA_V2;
use jet_client_native::{JetSimulationClient, SimulationClient};
use jet_instructions::airspace::derive_airspace;
use jet_instructions::fixed_term::derive_market_from_tenor;
use jet_instructions::test_service::{
    derive_pyth_price, derive_token_mint, token_update_pyth_price,
};
use jet_margin_pool::PoolFlags;
use jet_margin_sdk::ix_builder::test_service::derive_swap_pool;
use jet_margin_sdk::ix_builder::MarginConfigIxBuilder;
use jet_margin_sdk::solana::keypair::clone;
use jet_margin_sdk::solana::transaction::{InverseSendTransactionBuilder, SendTransactionBuilder};
use jet_margin_sdk::test_service::{
    init_environment, minimal_environment, AirspaceConfig, AirspaceTokenConfig, EnvironmentConfig,
    MarginPoolConfig, SwapPoolsConfig, TokenDescription,
};
use jet_margin_sdk::util::data::With;
use jet_metadata::TokenKind;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

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
    pub margin_config: MarginConfigIxBuilder,
    pub airspace_authority: Keypair,
    pub payer: Keypair,
    pub solana: SolanaTestContext,
}

impl std::fmt::Debug for MarginTestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarginTestContext")
            .field("airspace_authority", &self.airspace_authority)
            .field("payer", &self.payer)
            .finish()
    }
}

impl From<SolanaTestContext> for MarginTestContext {
    fn from(solana: SolanaTestContext) -> Self {
        let payer = Keypair::from_bytes(&solana.rpc.payer().to_bytes()).unwrap();
        let airspace_authority = solana.keygen.generate_key();
        let margin = MarginClient::new(
            solana.rpc.clone(),
            &airspace_authority.pubkey().to_string()[0..8],
            Some(clone(&airspace_authority)),
        );
        MarginTestContext {
            tokens: TokenManager::new(solana.clone()),
            margin_config: MarginConfigIxBuilder::new(
                margin.airspace(),
                solana.rpc.payer().pubkey(),
                Some(airspace_authority.pubkey()),
            ),
            margin,
            airspace_authority,
            rpc: solana.rpc.clone(),
            solana,
            payer,
        }
    }
}

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
        self.solana.create_wallet(sol_amount).await
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

pub struct TestContext {
    pub config: JetAppConfig,
    inner: SolanaTestContext,
}

impl TestContext {
    pub async fn new(name: &str, setup: &TestContextSetupInfo) -> Self {
        let mut seed = match cfg!(feature = "localnet") {
            false => name.to_owned(),
            true => format!("{name}_{}", rand::random::<u16>()),
        };

        seed.drain(0..seed.len().saturating_sub(24));

        let inner = SolanaTestContext::new(&seed).await;
        let setup_config = setup.to_config(&seed);

        let init_env_config = EnvironmentConfig {
            authority: inner.rpc.payer().pubkey(),
            tokens: setup_config
                .tokens
                .iter()
                .map(|(name, decimals, _)| TokenDescription {
                    symbol: name.to_string(),
                    name: name.to_string(),
                    precision: *decimals,
                    decimals: *decimals,
                })
                .collect(),
            swap_pools: SwapPoolsConfig {
                spl: setup_config
                    .spl_swap_pools
                    .iter()
                    .map(|(a, b)| format!("{a}/{b}"))
                    .collect(),
            },
            airspaces: vec![AirspaceConfig {
                name: seed.to_string(),
                is_restricted: setup.is_restricted,
                tokens: setup_config
                    .tokens
                    .iter()
                    .map(|(name, _, config)| (name.to_string(), config.clone()))
                    .collect(),
            }],
        };

        init_environment(&init_env_config, &Default::default())
            .unwrap()
            .send_and_confirm_condensed_in_order(&inner.rpc)
            .await
            .unwrap();

        Self {
            inner,
            config: setup_config.config,
        }
    }

    pub fn rpc(&self) -> &Arc<dyn SolanaRpcClient> {
        &self.inner.rpc
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair, Error> {
        jet_simulation::create_wallet(&self.inner.rpc, sol_amount * LAMPORTS_PER_SOL).await
    }

    pub async fn create_user(&self) -> Result<JetSimulationClient, Error> {
        let wallet = self.create_wallet(1_000).await?;
        let client = SimulationClient::new(self.inner.rpc.clone(), Some(wallet));

        Ok(JetSimulationClient::new(
            client,
            self.config.clone(),
            &self.config.airspaces[0].name,
        )?)
    }

    pub async fn update_price(&self, mint: &Pubkey, update: &PriceUpdate) -> Result<(), Error> {
        let ix = token_update_pyth_price(
            &self.rpc().payer().pubkey(),
            mint,
            update.price,
            update.confidence,
            update.exponent,
        );

        self.rpc().send_and_confirm(ix.into()).await?;
        Ok(())
    }

    pub async fn set_price(&self, mint: &Pubkey, price: f64, confidence: f64) -> Result<(), Error> {
        let exponent = -7;
        let one = 10_000_000.0;
        let price = (one * price).round() as i64;
        let confidence = (one * confidence).round() as i64;

        self.update_price(
            mint,
            &PriceUpdate {
                price,
                confidence,
                exponent,
            },
        )
        .await
    }
}

pub struct PriceUpdate {
    pub price: i64,
    pub confidence: i64,
    pub exponent: i32,
}

pub const DEFAULT_POOL_CONFIG: MarginPoolConfig = MarginPoolConfig {
    borrow_rate_0: 10,
    borrow_rate_1: 20,
    borrow_rate_2: 30,
    borrow_rate_3: 40,
    utilization_rate_1: 10,
    utilization_rate_2: 20,
    management_fee_rate: 10,
    flags: PoolFlags::ALLOW_LENDING.bits(),
};

pub fn default_test_setup() -> TestContextSetupInfo {
    TestContextSetupInfo {
        is_restricted: false,
        tokens: vec![
            (
                "TSOL",
                9,
                AirspaceTokenConfig {
                    collateral_weight: 100,
                    max_leverage: 20_00,
                    margin_pool_config: Some(DEFAULT_POOL_CONFIG),
                    fixed_term_markets: vec![],
                },
            ),
            (
                "USDC",
                6,
                AirspaceTokenConfig {
                    collateral_weight: 100,
                    max_leverage: 20_00,
                    margin_pool_config: Some(DEFAULT_POOL_CONFIG),
                    fixed_term_markets: vec![],
                },
            ),
        ],
        spl_swap_pools: vec!["TSOL/USDC"],
    }
}

#[derive(Default, Clone)]
pub struct TestContextSetupInfo {
    pub is_restricted: bool,
    pub tokens: Vec<(&'static str, u8, AirspaceTokenConfig)>,
    pub spl_swap_pools: Vec<&'static str>,
}

struct SetupOutput {
    config: JetAppConfig,
    tokens: Vec<(String, u8, AirspaceTokenConfig)>,
    spl_swap_pools: Vec<(String, String)>,
}

impl TestContextSetupInfo {
    fn to_config(&self, seed: &str) -> SetupOutput {
        let airspace = derive_airspace(seed);
        let tokens = self
            .tokens
            .iter()
            .map(|(s, d, c)| (format!("{seed}-{s}"), *d, c.clone()))
            .collect::<Vec<_>>();

        let spl_swap_pools = self
            .spl_swap_pools
            .iter()
            .map(|pair_string| {
                let (name_a, name_b) = pair_string.split_once('/').unwrap();
                let token_a_name = format!("{seed}-{name_a}");
                let token_b_name = format!("{seed}-{name_b}");

                (token_a_name, token_b_name)
            })
            .collect::<Vec<_>>();

        let config = JetAppConfig {
            tokens: tokens
                .iter()
                .map(|(name, decimals, _)| {
                    let mint = derive_token_mint(name);
                    TokenInfo {
                        symbol: name.clone(),
                        name: name.clone(),
                        precision: *decimals,
                        decimals: *decimals,
                        oracle: derive_pyth_price(&mint),
                        mint,
                    }
                })
                .collect(),
            airspaces: vec![AirspaceInfo {
                name: seed.to_string(),
                tokens: tokens.iter().map(|(name, _, _)| name.clone()).collect(),
                fixed_term_markets: tokens
                    .iter()
                    .flat_map(|(name, _, c)| {
                        let token = derive_token_mint(name);

                        c.fixed_term_markets.iter().map(move |m| {
                            derive_market_from_tenor(&airspace, &token, m.borrow_tenor)
                        })
                    })
                    .collect(),
            }],
            exchanges: spl_swap_pools
                .iter()
                .map(|(name_a, name_b)| {
                    let token_a = derive_token_mint(name_a);
                    let token_b = derive_token_mint(name_b);

                    DexInfo {
                        program: ORCA_V2,
                        address: derive_swap_pool(&token_a, &token_b).state,
                        token_a,
                        token_b,
                    }
                })
                .collect(),
        };

        SetupOutput {
            config,
            tokens,
            spl_swap_pools,
        }
    }
}

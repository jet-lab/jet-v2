use std::sync::Arc;

use anyhow::Error;
use jet_margin_sdk::ix_builder::MarginConfigIxBuilder;
use jet_margin_sdk::solana::keypair::clone;
use jet_margin_sdk::solana::transaction::InverseSendTransactionBuilder;
use jet_margin_sdk::util::data::With;

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

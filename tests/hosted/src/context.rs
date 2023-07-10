use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;

use solana_sdk::pubkey::Pubkey;

use jet_client::config::JetAppConfig;
use jet_client::{JetClient, NetworkKind};
use jet_environment::config::{
    AirspaceConfig, DexConfig, EnvironmentConfig, TokenDescription, DEFAULT_MARGIN_ADAPTERS,
};
use jet_margin_pool::MarginPoolConfig;
use jet_metadata::TokenKind;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::environment::TestToken;
use crate::TestDefault;

pub mod admin;
pub mod margin;
pub mod margin_account;
pub mod token;
pub use margin::MarginTestContext;

/// Instantiate an Arc<MarginTestContext>
///
/// If you don't provide a name, gets the name of the current function name and
/// uses it to create a test context. Only use this way when called directly in
/// the test function. If you want to call this in a helper function, pass a
/// name that is unique to the individual test.
#[macro_export]
macro_rules! margin_test_context {
    () => {
        $crate::margin_test_context!(&$crate::fn_name_and_try_num!())
    };
    ($name:expr) => {
        std::sync::Arc::new(
            $crate::context::MarginTestContext::new($name)
                .await
                .unwrap()
                .and_init(&Default::default())
                .await
                .unwrap(),
        )
    };
}

/// Instantiate a TestContext  
/// Uses struct-like syntax. Fields may be omitted to use the default.
/// ```ignore
/// test_context! {
///     name: &str,
///     setup: &TestContextSetupInfo,
/// };
/// test_context!();
/// test_context!(setup, name);
/// ```
/// - name: Default gets the name of the current function name and uses it to
///         create a test context. Only use this way when called directly in the
///         test function. If you want to call this in a helper function, pass a
///         name to the helper function that is unique to the individual test
///         that called the helper function.
///
/// - setup: see the TestDefault implementation.
#[macro_export]
macro_rules! test_context {
    (
        $(name $(: $name:expr)? ,)?
        $(setup $(: $setup:expr)? )?
        $(,)?
    ) => {
        $crate::context::TestContext::new(
            $crate::first!($($($name)?, name)?, &$crate::fn_name_and_try_num!()),
            $crate::first!($($($setup)?, setup)?, &$crate::test_default()),
        )
            .await
            .unwrap()
    };
    (
        $(setup $(: $setup:expr)? ,)?
        $(name $(: $name:expr)? )?
        $(,)?
    ) => {
        $crate::test_context!{
            $(name: $($name)?,)?
            $(setup: $($setup)?,)?
        }
    };
}

/// Returns the first item.
///
/// Useful within in macro definitions where it is uncertain whether an item
/// will be expanded to anything.
///
/// Delimit items with ",". Extra commas are allowed anywhere.
/// ```
/// use hosted_tests::first;
/// let (one, two, three) = (1, 2, 3);
/// assert_eq!(1, first!(one, two, three,,));
/// assert_eq!(2, first!(, two,,, three));
/// ```
#[macro_export]
macro_rules! first {
    ($(,)* $item:expr $($(,)+ $default:expr)* $(,)*) => {
        $item
    };
}

pub struct MarginPoolSetupInfo {
    pub token: Pubkey,
    pub fee_vault: Pubkey,
    pub kind: TokenKind,
    pub weight: u16,
    pub config: MarginPoolConfig,
}

/// Sets up a comprehensive test environment using the  with tokens, pools, markets, etc.
/// as defined by the provided configuration.
pub struct TestContext {
    pub config: JetAppConfig,
    pub inner: MarginTestContext,
}

impl TestContext {
    pub async fn new(name: &str, setup: &TestContextSetupInfo) -> Result<Self, Error> {
        let inner = MarginTestContext::new(name).await?;
        let config = inner.init_environment(setup).await?;

        Ok(Self { config, inner })
    }

    pub fn rpc(&self) -> &Arc<dyn SolanaRpcClient> {
        &self.inner.solana.rpc
    }

    pub async fn create_user(&self) -> Result<JetClient, Error> {
        let wallet = Rc::new(self.inner.create_margin_user(1_000).await?.signer);

        Ok(JetClient::new(
            self.inner.solana.rpc2.clone(),
            wallet,
            self.config.clone(),
            &self.config.airspaces[0].name,
        )?)
    }

    pub async fn create_lookup_registry(&self, addresses: &[Pubkey]) -> Result<Pubkey, Error> {
        self.inner.create_lookup_registry(addresses).await
    }
}

#[derive(Default, Clone)]
pub struct TestContextSetupInfo {
    pub is_restricted: bool,
    pub tokens: Vec<TokenDescription>,
    pub dexes: Vec<(&'static str, &'static str)>,
}

impl TestDefault for TestContextSetupInfo {
    fn test_default() -> Self {
        TestContextSetupInfo {
            is_restricted: false,
            tokens: vec![
                TestToken::with_pool("TSOL").into(),
                TestToken::with_pool("USDC").into(),
            ],
            dexes: vec![("spl-swap", "TSOL/USDC")],
        }
    }
}

impl TestContextSetupInfo {
    fn to_config(&self, airspace_name: &str, payer: Pubkey, crank: Pubkey) -> EnvironmentConfig {
        let tokens = self
            .tokens
            .iter()
            .map(|t| TokenDescription {
                name: format!("{airspace_name}-{}", &t.name),
                ..t.clone()
            })
            .collect::<Vec<_>>();

        let dexes = self
            .dexes
            .iter()
            .map(|(program, pair_string)| {
                let (name_a, name_b) = pair_string.split_once('/').unwrap();
                let token_a_name = format!("{airspace_name}-{name_a}");
                let token_b_name = format!("{airspace_name}-{name_b}");

                (program, (token_a_name, token_b_name))
            })
            .collect::<Vec<_>>();

        let env_config = EnvironmentConfig {
            network: NetworkKind::Localnet,
            margin_adapters: DEFAULT_MARGIN_ADAPTERS.to_vec(),
            oracle_authority: Some(payer),
            exchanges: dexes
                .iter()
                .map(|(program, (a, b))| DexConfig {
                    program: program.to_string(),
                    description: None,
                    state: None,
                    base: a.clone(),
                    quote: b.clone(),
                })
                .collect(),
            airspaces: vec![AirspaceConfig {
                name: airspace_name.to_string(),
                is_restricted: self.is_restricted,
                tokens,
                cranks: vec![crank],
                lookup_registry_authority: None,
            }],
        };

        env_config
    }
}

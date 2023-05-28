//! Defines the MarginTestContext data structure

use std::sync::Arc;

use jet_client::NetworkKind;
use jet_environment::builder::{Builder, ProposalExecution};
use jet_solana_client::util::keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_instructions::airspace::{derive_airspace, AirspaceIxBuilder};
use jet_instructions::margin::MarginConfigIxBuilder;
use jet_margin_sdk::solana::keypair::KeypairExt;
use jet_simulation::hash;
use jet_simulation::solana_rpc_api::SolanaRpcClient;

use crate::runtime::SolanaTestContext;
use crate::{margin::MarginClient, tokens::TokenManager};

use super::TestContextSetupInfo;

/// Data structure containing the minimal state needed to run client-side
/// integration tests for the entire on-chain margin ecosystem:
/// - solana test context.
/// - keys that are necessary to administrate an airspace
///
/// This should not become a bucket for random test state. It should be
/// immutable, aside from the internals of SolanaTestContext.
///
/// Reduces test boilerplate via:
/// - Helper methods to set up and administer the test environment, such as
///   `init_airspace` and `create_user`
/// - Getters for other types such as clients and ix builders.
pub struct MarginTestContext {
    pub solana: SolanaTestContext,
    pub airspace: Pubkey,
    /// Seed used to generate the airspace address
    pub airspace_name: String,
    /// Account authorized by the airspaces program to register and administrate
    /// margin adapters.
    pub airspace_authority: Keypair,
    /// Account authorized in adapter programs to execute privileged crank
    /// functions such as consume_events and settle in fixed term.
    pub crank: Keypair,
}

/// Constructors
impl MarginTestContext {
    pub async fn new(name: &str) -> anyhow::Result<Self> {
        let solana = SolanaTestContext::new(name).await;
        let airspace_name = airspace_name(name);

        Ok(Self {
            airspace: derive_airspace(&airspace_name),
            airspace_name,
            airspace_authority: solana.generate_key(),
            crank: solana.create_wallet(10).await?,
            solana,
        })
    }

    pub async fn and_init(self, setup: &TestContextSetupInfo) -> anyhow::Result<Self> {
        self.init_environment(setup).await?;
        Ok(self)
    }
}

/// Getters  
/// - reorganize the contained data into another type.
impl MarginTestContext {
    pub fn payer(&self) -> &Keypair {
        self.solana.rpc.payer()
    }

    pub fn rpc(&self) -> Arc<dyn SolanaRpcClient> {
        self.solana.rpc.clone()
    }

    pub fn margin_config_ix(&self) -> MarginConfigIxBuilder {
        MarginConfigIxBuilder::new(
            self.airspace,
            self.payer().pubkey(),
            Some(self.airspace_authority.pubkey()),
        )
    }

    pub fn airspace_ix(&self) -> AirspaceIxBuilder {
        AirspaceIxBuilder::new(
            &self.airspace_name,
            self.payer().pubkey(),
            self.airspace_authority.pubkey(),
        )
    }

    pub fn margin_client(&self) -> MarginClient {
        MarginClient::new(
            self.solana.rpc.clone(),
            &self.airspace_name,
            Some(self.airspace_authority.clone()),
        )
    }

    /// This manages oracles with the metadata program. Some other code uses the
    /// test service. These two approaches are not compatible.
    pub fn tokens(&self) -> TokenManager {
        TokenManager::new(self.solana.clone())
    }

    pub fn env_builder(&self) -> Builder {
        let signer = Arc::new(keypair::clone(self.solana.rpc.payer()));
        let authority = self.airspace_authority.pubkey();

        Builder::new_infallible(
            self.solana.rpc2.clone(),
            signer,
            ProposalExecution::Direct { authority },
            NetworkKind::Localnet,
        )
    }
}

/// Airspace names are not allowed to exceed 24 characters, so the test name
/// must be truncated.  
/// Derived from a fully qualified path of a test function, this includes human
/// readable information about both the module name and the function name, plus a unique number
fn airspace_name(test_name: &str) -> String {
    let len = test_name.len();
    if len <= 24 {
        test_name.to_owned()
    } else {
        let uniq = ((hash(test_name) % 256) as u8).to_string();
        format!(
            "{}.{uniq}.{}",
            test_name[0..2].to_owned(),
            test_name[len - 20 + uniq.len()..len].to_owned()
        )
    }
}

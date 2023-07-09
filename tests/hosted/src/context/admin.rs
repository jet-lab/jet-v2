//! General Airspace administration helper functions for tests.  
//! See token.rs for token-specific administration.

use anyhow::Result;
use jet_client::config::JetAppConfig;
use jet_client::NetworkKind;
use jet_environment::builder::{
    configure_environment, configure_market_for_token, token_context, PlanInstructions,
};
use jet_instructions::fixed_term::FixedTermIxBuilder;
use jet_instructions::test_service::derive_token_mint;
use jet_tools::lookup_tables::create_lookup_tables;
use lookup_table_registry_client::instructions::InstructionBuilder;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};

use crate::actions::Token;
use crate::margin::MarginUser;
use jet_environment::config::{FixedTermMarketConfig, TokenDescription};

use jet_margin_sdk::solana::transaction::{InverseSendTransactionBuilder, TransactionBuilderExt};
use jet_simulation::Keygen;
use jet_solana_client::transaction::WithSigner;

use super::{MarginTestContext, TestContextSetupInfo};

/// General margin or airspace administration
impl MarginTestContext {
    /// Create the airspace plus all tokens, pools, swaps, and markets.
    ///
    /// setup options:
    /// - Default::default(): a blank airspace
    /// - TestDefault::test_default(): airspace with two tokens and their pools
    pub async fn init_environment(&self, setup: &TestContextSetupInfo) -> Result<JetAppConfig> {
        let setup_config = setup.to_config(
            &self.airspace_name,
            self.solana.rpc.payer().pubkey(),
            self.crank.pubkey(),
            self.airspace_authority.pubkey(),
        );
        let mut builder = self.env_builder();
        configure_environment(&mut builder, &setup_config.env_config)
            .await
            .unwrap();
        self.execute_plan(builder.build()).await?;

        Ok(setup_config.app_config)
    }

    pub(super) async fn execute_plan(&self, plan: PlanInstructions) -> Result<()> {
        for setup in plan.setup {
            setup.send_and_confirm_condensed(&self.solana.rpc).await?;
        }

        plan.propose
            .into_iter()
            .map(|tx| tx.with_signer(&self.airspace_authority))
            .collect::<Vec<_>>()
            .send_and_confirm_condensed_in_order(&self.solana.rpc)
            .await?;

        create_lookup_tables(
            &self.solana.rpc2,
            self.payer(),
            self.payer(),
            &plan.lookup_setup,
        )
        .await?;

        Ok(())
    }

    pub fn generate_key(&self) -> Keypair {
        self.solana.keygen.generate_key()
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair> {
        self.solana.create_wallet(sol_amount).await
    }

    pub async fn issue_permit(&self, user: Pubkey) -> Result<Signature> {
        self.airspace_ix()
            .permit_create(user)
            .with_signer(&self.airspace_authority)
            .send_and_confirm(&self.solana.rpc)
            .await
    }

    pub async fn create_margin_user(&self, sol_amount: u64) -> Result<MarginUser> {
        let wallet = self.solana.create_wallet(sol_amount).await?;
        self.issue_permit(wallet.pubkey()).await?;
        self.margin_client().user(&wallet, 0).created().await
    }

    /// Generate a new wallet keypair for a liquidator with the pubkey that
    /// stores the [LiquidatorMetadata]
    pub async fn create_liquidator(&self, sol_amount: u64) -> Result<Keypair> {
        let liquidator = self.solana.create_wallet(sol_amount).await?;

        self.margin_client()
            .set_liquidator_metadata(liquidator.pubkey(), true)
            .await?;
        Ok(liquidator)
    }
}

/// Fixed term
impl MarginTestContext {
    pub async fn create_fixed_term_market(
        &self,
        underlying_token_mint: TokenDescription,
        config: FixedTermMarketConfig,
    ) -> Result<FixedTermIxBuilder> {
        let mut builder = self.env_builder();
        let token_context = token_context(
            NetworkKind::Localnet,
            &self.airspace,
            self.payer().pubkey(),
            &underlying_token_mint,
        )
        .unwrap();
        let ix_builder = configure_market_for_token(
            &mut builder,
            &[self.crank.pubkey()],
            &token_context,
            &config,
        )
        .await
        .unwrap();

        self.execute_plan(builder.build()).await?;

        Ok(ix_builder)
    }
}

/// Lookup tables
impl MarginTestContext {
    pub async fn create_lookup_registry(&self, addresses: &[Pubkey]) -> Result<Pubkey> {
        // As a convenience, take advantage of addresses < 256.
        assert!(
            addresses.len() < 256,
            "Expecting addresses to all fit a single lookup table"
        );

        let authority = self.airspace_authority.pubkey();
        let payer = self.payer().pubkey();
        let builder = InstructionBuilder::new(authority, payer);
        let registry_address = builder.registry_address();
        // Create the registry
        let init_registry_ix = builder.init_registry();
        let tx = self
            .rpc()
            .create_transaction(&[&self.airspace_authority], &[init_registry_ix])
            .await?;
        self.rpc().send_transaction(&tx).await?;

        // Create a lookup table and add addresses to it
        // First wait for a few slots
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let recent_slot = self.solana.rpc.get_slot(None).await?;
        let (create_lookup_table_ix, lookup_table) = builder.create_lookup_table(recent_slot, 0);
        let tx = self
            .rpc()
            .create_transaction(&[&self.airspace_authority], &[create_lookup_table_ix])
            .await?;
        self.rpc().send_transaction(&tx).await?;

        for chunk in addresses.chunks(20) {
            let ix = builder.append_to_lookup_table(lookup_table, chunk, 0);
            let tx = self
                .rpc()
                .create_transaction(&[&self.airspace_authority], &[ix])
                .await?;
            self.rpc().send_transaction(&tx).await?;
        }

        Ok(registry_address)
    }
}

impl From<TokenDescription> for Token {
    fn from(value: TokenDescription) -> Self {
        Self {
            mint: derive_token_mint(&value.name),
            decimals: value.decimals.unwrap(),
        }
    }
}

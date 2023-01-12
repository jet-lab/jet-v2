use std::path::PathBuf;

use anyhow::{Result};
use jet_environment::builder::{configure_environment, Builder};
use jet_program_common::{GOVERNOR_DEVNET, GOVERNOR_MAINNET};
use solana_sdk::signer::Signer;

use crate::{
    client::{Client, NetworkKind, Plan},
};

pub async fn process_apply(client: &Client, config_path: PathBuf) -> Result<Plan> {
    let config = jet_environment::config::read_env_config_dir(&config_path)?;

    let authority = match client.network_kind {
        NetworkKind::Mainnet => GOVERNOR_MAINNET,
        NetworkKind::Devnet => GOVERNOR_DEVNET,
        NetworkKind::Localnet => client.signer()?,
    };

    let mut builder = Builder::new(client.network_interface(), authority)
        .await
        .unwrap();

    configure_environment(&mut builder, &config).await?;

    let blueprint = builder.build();
    let mut plan = client.plan()?;

    for setup_tx in blueprint.setup {
        let signers = setup_tx
            .signers
            .into_iter()
            .map(|k| Box::new(k) as Box<dyn Signer>);

        plan = plan.instructions(signers, [""], setup_tx.instructions);
    }

    for propose_tx in blueprint.propose {
        plan = plan.instructions([], [""], propose_tx.instructions);
    }

    Ok(plan.build())
}

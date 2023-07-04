use std::path::PathBuf;

use anchor_lang::prelude::Pubkey;
use anyhow::{bail, Result};
use jet_environment::builder::{
    configure_environment, Builder, ProposalContext, ProposalExecution,
};
use jet_program_common::{GOVERNOR_DEVNET, GOVERNOR_MAINNET};
use jet_solana_client::util::keypair::clone;
use solana_sdk::signer::Signer;

use crate::{
    client::{Client, NetworkKind, Plan},
    governance::{get_proposal_state, JET_GOVERNANCE_PROGRAM},
};

pub async fn process_apply(
    client: &Client,
    config_path: PathBuf,
    proposal: Option<Pubkey>,
    proposal_option: u8,
) -> Result<Plan> {
    let config = jet_environment::config::read_env_config_dir(&config_path)?;

    let authority = match client.network_kind {
        NetworkKind::Mainnet => GOVERNOR_MAINNET,
        NetworkKind::Devnet => GOVERNOR_DEVNET,
        NetworkKind::Localnet => client.signer()?,
    };

    let proposal_execution = match (client.network_kind, proposal) {
        (NetworkKind::Localnet, None) => ProposalExecution::Direct { authority },
        (_, None) => bail!("must target a proposal for effecting changes on public networks"),
        (_, Some(proposal_id)) => {
            let proposal = get_proposal_state(client, &proposal_id).await?;
            let tx_next_index = proposal.options[proposal_option as usize].transactions_next_index;

            if proposal.governance != authority {
                bail!(
                    "the proposal does not assume the right authority, got {} but expected {}",
                    proposal.governance,
                    authority
                );
            }

            ProposalExecution::Governance(ProposalContext {
                program: JET_GOVERNANCE_PROGRAM,
                proposal: proposal_id,
                governance: proposal.governance,
                option: proposal_option,
                proposal_owner_record: proposal.token_owner_record,
                tx_next_index,
            })
        }
    };

    let mut builder = Builder::new(
        client.network_interface(),
        client.signer_ref()?,
        proposal_execution,
    )
    .await
    .unwrap();

    configure_environment(&mut builder, &config).await.unwrap();

    let blueprint = builder.build();
    let mut plan = client.plan()?;

    for setup in blueprint.setup {
        for setup_tx in setup {
            let signers = setup_tx
                .signers
                .into_iter()
                .map(|k| Box::new(clone(&k)) as Box<dyn Signer>);

            plan = plan.instructions(signers, [""], setup_tx.instructions);
        }
    }

    for propose_tx in blueprint.propose {
        plan = plan.instructions([], [""], propose_tx.instructions);
    }

    Ok(plan.build())
}

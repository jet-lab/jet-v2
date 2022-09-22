use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use solana_account_decoder::parse_bpf_loader::{
    parse_bpf_upgradeable_loader, BpfUpgradeableLoaderAccountType, UiBuffer, UiProgram,
    UiProgramData,
};
use solana_sdk::{bpf_loader_upgradeable, pubkey::Pubkey};

use crate::client::{Client, Plan};

pub async fn process_deploy(
    client: &Client,
    proposal_id: Pubkey,
    program_id: Pubkey,
    buffer_id: Pubkey,
) -> Result<Plan> {
    let proposal = crate::governance::get_proposal_state(client, &proposal_id).await?;
    let program_authority = get_program_authority(client, &program_id).await?;
    let buffer_authority = get_buffer_authority(client, &buffer_id).await?;

    // sanity check upgrade authority is right
    if program_authority != proposal.governance {
        bail!("proposal {proposal_id} is not proposing for the correct authority: {program_authority}");
    }

    // sanity check buffer authority is also set correctly
    if buffer_authority != proposal.governance {
        bail!("the buffer {buffer_id} does not have its authority set to the governing address for the proposal: {}", proposal.governance)
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("deploy {program_id} with buffer {buffer_id}")],
            [bpf_loader_upgradeable::upgrade(
                &program_id,
                &buffer_id,
                &program_authority,
                &client.signer()?,
            )],
        )
        .build())
}

async fn get_buffer_authority(client: &Client, buffer_id: &Pubkey) -> Result<Pubkey> {
    let buffer_account_bytes = client.rpc().get_account_data(buffer_id).await?;
    match parse_bpf_upgradeable_loader(&buffer_account_bytes)? {
        BpfUpgradeableLoaderAccountType::Buffer(UiBuffer { authority, .. }) => {
            Ok(Pubkey::from_str(&authority.ok_or_else(|| {
                anyhow!("buffer {buffer_id} has no authority")
            })?)?)
        }
        unexpected => bail!("unexpected buffer account for {buffer_id}: {unexpected:?}"),
    }
}

async fn get_program_authority(client: &Client, program_id: &Pubkey) -> Result<Pubkey> {
    let program_account_bytes = client.rpc().get_account_data(program_id).await?;
    let program_data_account_id = match parse_bpf_upgradeable_loader(&program_account_bytes)? {
        BpfUpgradeableLoaderAccountType::Program(UiProgram { program_data }) => {
            Pubkey::from_str(&program_data)?
        }
        unexpected => bail!("unexpected account type for program {program_id}: {unexpected:?}"),
    };

    let program_data_bytes = client
        .rpc()
        .get_account_data(&program_data_account_id)
        .await?;

    let program_authority = match parse_bpf_upgradeable_loader(&program_data_bytes)? {
        BpfUpgradeableLoaderAccountType::ProgramData(UiProgramData { authority, .. }) => {
            Pubkey::from_str(
                &authority.ok_or_else(|| anyhow!("program {program_id} cannot be upgraded"))?,
            )?
        }
        unexpected => bail!("unexpected program data for {program_id}: {unexpected:?}"),
    };

    Ok(program_authority)
}

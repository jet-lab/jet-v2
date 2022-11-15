use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use spl_governance::state::proposal::get_proposal_address;
use spl_governance::state::proposal::VoteType;
use spl_governance::state::vote_record::{Vote, VoteChoice};

use crate::client::{Client, Plan};
use crate::governance::{find_user_owner_record, JET_GOVERNANCE_PROGRAM, JET_STAKING_PROGRAM};

pub async fn process_proposal_create(
    client: &Client,
    gov_name_or_pubkey: &str,
    title: String,
    description: String,
) -> Result<Plan> {
    let governance_address =
        crate::governance::get_governance_address_from_user_string(gov_name_or_pubkey)?;
    let (governance, realm) =
        crate::governance::get_governance_and_realm(client, &governance_address).await?;

    let proposal_owner_record = find_user_owner_record(
        client,
        &governance.realm,
        realm.config.council_mint.as_ref().unwrap(),
    )
    .await?;

    let proposal_address = get_proposal_address(
        &JET_GOVERNANCE_PROGRAM,
        &governance_address,
        realm.config.council_mint.as_ref().unwrap(),
        &governance.proposals_count.to_le_bytes(),
    );

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!(
                "create proposal for {gov_name_or_pubkey} ({governance_address}): {proposal_address}"
            )],
            [spl_governance::instruction::create_proposal(
                &JET_GOVERNANCE_PROGRAM,
                &governance_address,
                &proposal_owner_record,
                &client.signer()?,
                &client.signer()?,
                None,
                &governance.realm,
                title,
                description,
                realm.config.council_mint.as_ref().unwrap(),
                VoteType::SingleChoice,
                vec!["Approve".to_owned()],
                true,
                governance.proposals_count,
            )],
        )
        .build())
}

pub async fn process_proposal_sign_off(client: &Client, proposal_address: Pubkey) -> Result<Plan> {
    let proposal = crate::governance::get_proposal_state(client, &proposal_address).await?;
    let (governance, _) =
        crate::governance::get_governance_and_realm(client, &proposal.governance).await?;

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("sign off proposal {proposal_address}")],
            [spl_governance::instruction::sign_off_proposal(
                &JET_GOVERNANCE_PROGRAM,
                &governance.realm,
                &proposal.governance,
                &proposal_address,
                &client.signer()?,
                Some(&proposal.token_owner_record),
            )],
        )
        .build())
}

pub async fn process_proposal_finalize(client: &Client, proposal_address: Pubkey) -> Result<Plan> {
    let proposal = crate::governance::get_proposal_state(client, &proposal_address).await?;
    let (governance, _) =
        crate::governance::get_governance_and_realm(client, &proposal.governance).await?;

    let (jet_max_weight, _) = Pubkey::find_program_address(
        &[governance.realm.as_ref(), b"max-vote-weight-record"],
        &JET_STAKING_PROGRAM,
    );

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("finalize proposal {proposal_address}")],
            [spl_governance::instruction::finalize_vote(
                &JET_GOVERNANCE_PROGRAM,
                &governance.realm,
                &proposal.governance,
                &proposal_address,
                &proposal.token_owner_record,
                &proposal.governing_token_mint,
                Some(jet_max_weight),
            )],
        )
        .build())
}

pub async fn process_proposal_approve(client: &Client, proposal_address: Pubkey) -> Result<Plan> {
    let proposal = crate::governance::get_proposal_state(client, &proposal_address).await?;
    let (governance, realm) =
        crate::governance::get_governance_and_realm(client, &proposal.governance).await?;
    let voter_token_owner_record = find_user_owner_record(
        client,
        &governance.realm,
        realm.config.council_mint.as_ref().unwrap(),
    )
    .await?;

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("approve proposal {proposal_address}")],
            [spl_governance::instruction::cast_vote(
                &JET_GOVERNANCE_PROGRAM,
                &governance.realm,
                &proposal.governance,
                &proposal_address,
                &proposal.token_owner_record,
                &voter_token_owner_record,
                &client.signer()?,
                realm.config.council_mint.as_ref().unwrap(),
                &client.signer()?,
                None,
                None,
                Vote::Approve(vec![VoteChoice {
                    rank: 0,
                    weight_percentage: 100,
                }]),
            )],
        )
        .build())
}

pub async fn process_proposal_execute(client: &Client, proposal_address: Pubkey) -> Result<Plan> {
    let to_execute = crate::governance::get_execute_instructions(client, proposal_address).await?;
    let mut plan = client.plan()?;

    for (idx, ix) in to_execute {
        plan = plan.instructions([], [format!("Tx {idx}")], [ix]);
    }

    Ok(plan.build())
}

pub async fn process_proposal_clear_instructions(
    client: &Client,
    proposal_address: Pubkey,
) -> Result<Plan> {
    let to_execute =
        crate::governance::get_clear_tx_instructions(client, &proposal_address).await?;
    let mut plan = client.plan()?;

    for (desc, ix) in to_execute {
        plan = plan.instructions([], [desc], [ix]);
    }

    Ok(plan.build())
}

pub async fn process_proposal_inspect(client: &Client, proposal_address: Pubkey) -> Result<Plan> {
    crate::governance::inspect_proposal_instructions(client, proposal_address).await?;
    Ok(Plan::new())
}

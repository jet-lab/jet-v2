use solana_sdk::{pubkey::Pubkey, instruction::Instruction, clock::SECONDS_PER_DAY};

use jet_instructions::{staking::STAKING_PROGRAM, test_service::derive_token_mint};
use jet_program_common::GOVERNANCE_PROGRAM;
use jet_solana_client::network::NetworkKind;
use spl_governance::state::{enums::{MintMaxVoteWeightSource, VoteTipping, VoteThresholdPercentage}, governance::GovernanceConfig};

use crate::config::TokenDescription;

use super::{create_test_tokens, Builder, BuilderError, SetupPhase};

pub async fn create_governance_system(
    builder: &mut Builder,
    oracle_authority: &Pubkey,
) -> Result<(), BuilderError> {
    if builder.network != NetworkKind::Localnet {
        return Ok(());
    }

    create_governance_token(builder, oracle_authority).await?;

    Ok(())
}

fn create_governance_realm_ix(payer: &Pubkey, authority: &Pubkey) -> Instruction {
    let token_mint = derive_token_mint("JET");

    spl_governance::instruction::create_realm(
        &GOVERNANCE_PROGRAM,
        authority,
        &token_mint,
        payer,
        None,
        Some(STAKING_PROGRAM),
        Some(STAKING_PROGRAM),
        "JET".to_string(),
        1_000_000_000,
        MintMaxVoteWeightSource::FULL_SUPPLY_FRACTION,
    )
}

fn create_governance_account(payer: &Pubkey, authority: &Pubkey, realm: &Pubkey) -> Instruction {
    let token_mint = derive_token_mint("JET");

    spl_governance::instruction::create_governance(
        &GOVERNANCE_PROGRAM,
        realm,
        None,
        authority,
        payer,
        authority,
        None,
        GovernanceConfig {
            vote_tipping: VoteTipping::Early,
            max_voting_time: SECONDS_PER_DAY as u32,
            vote_threshold_percentage: VoteThresholdPercentage::YesVote(50),
            min_community_weight_to_create_proposal: 
        }
    )
}

async fn create_governance_token(
    builder: &mut Builder,
    oracle_authority: &Pubkey,
) -> Result<(), BuilderError> {
    create_test_tokens(
        builder,
        oracle_authority,
        [&TokenDescription {
            name: "JET".to_string(),
            symbol: "JET".to_string(),
            decimals: Some(9),
            precision: 4,
            max_leverage: 0,
            collateral_weight: 0,
            ..Default::default()
        }],
    )
    .await
}

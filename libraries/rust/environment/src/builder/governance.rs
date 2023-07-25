use solana_sdk::{clock::SECONDS_PER_DAY, instruction::Instruction, pubkey::Pubkey};

use jet_instructions::{
    staking::{self, PoolConfig, STAKING_PROGRAM},
    test_service::derive_token_mint,
};
use jet_program_common::GOVERNANCE_PROGRAM;
use jet_solana_client::{network::NetworkKind, transaction::TransactionBuilder};
use spl_governance::state::{
    enums::{VoteTipping, VoteThreshold, MintMaxVoterWeightSource},
    governance::GovernanceConfig,
    realm::{get_realm_address, GoverningTokenConfigAccountArgs},
};

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

    builder.setup(
        SetupPhase::TokenAccounts,
        [TransactionBuilder {
            instructions: vec![
                create_governance_realm_ix(&builder.payer(), &builder.payer()),
                create_governance_account(&builder.payer(), &builder.payer()),
                create_stake_pool(&builder.payer(), &builder.payer()),
            ],
            signers: vec![],
        }],
    );

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
        Some(GoverningTokenConfigAccountArgs {
            voter_weight_addin: Some(STAKING_PROGRAM),
            max_voter_weight_addin: Some(STAKING_PROGRAM),
            ..Default::default()
        }),
        None,
        "JET".to_string(),
        1_000_000_000,
        MintMaxVoterWeightSource::FULL_SUPPLY_FRACTION,
    )
}

fn create_governance_account(payer: &Pubkey, authority: &Pubkey) -> Instruction {
    let token_mint = derive_token_mint("JET");
    let realm = get_realm_address(&GOVERNANCE_PROGRAM, "JET");

    spl_governance::instruction::create_governance(
        &GOVERNANCE_PROGRAM,
        &realm,
        Some(&token_mint),
        authority,
        payer,
        authority,
        None,
        GovernanceConfig {
            min_community_weight_to_create_proposal: 1_000_000_000,
            min_transaction_hold_up_time: 0,
            min_council_weight_to_create_proposal: 1,
            community_veto_vote_threshold: VoteThreshold::YesVotePercentage(10),
            community_vote_threshold: VoteThreshold::YesVotePercentage(10),
            community_vote_tipping: VoteTipping::Early,
            council_veto_vote_threshold: VoteThreshold::YesVotePercentage(10),
            council_vote_threshold: VoteThreshold::YesVotePercentage(10),
            council_vote_tipping: VoteTipping::Early,
            deposit_exempt_proposal_count: 0,
            voting_base_time: SECONDS_PER_DAY as u32,
            voting_cool_off_time: 0,
        },
    )
}

fn create_stake_pool(payer: &Pubkey, authority: &Pubkey) -> Instruction {
    let token_mint = derive_token_mint("JET");
    let realm = get_realm_address(&GOVERNANCE_PROGRAM, "JET");

    staking::init_pool(
        *payer,
        *authority,
        token_mint,
        "JET",
        &PoolConfig {
            unbond_period: 0,
            governance_realm: realm,
        },
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

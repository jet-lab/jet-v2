use std::collections::HashMap;

use anchor_lang::AnchorDeserialize;
use solana_sdk::pubkey::Pubkey;

use spl_governance::state::{
    enums::GovernanceAccountType,
    governance::GovernanceV2,
    proposal::ProposalV2,
    realm::{get_realm_address, RealmV2},
    token_owner_record::get_token_owner_record_address,
    vote_record::{get_vote_record_address, VoteRecordV2},
};

use jet_instructions::staking::{derive_stake_account, derive_stake_pool};
use jet_staking::state::{StakeAccount, StakePool};

use jet_program_common::{GOVERNANCE_PROGRAM, GOVERNANCE_REALM_DAO};
use jet_solana_client::rpc::{AccountFilter, SolanaRpcExtra};

use super::AccountStates;
use crate::{bail, ClientResult};

const DEFAULT_STAKE_SEED: &'static str = "jetgov";

pub struct RealmInfo {
    pub state: RealmV2,
    pub governances: HashMap<Pubkey, GovernanceV2>,
    pub proposals: HashMap<Pubkey, ProposalV2>,
    pub votes: HashMap<Pubkey, VoteRecordV2>,
}

pub async fn sync(states: &AccountStates) -> ClientResult<()> {
    sync_realm(states).await?;
    sync_stake(states).await?;
    Ok(())
}

pub async fn sync_realm(states: &AccountStates) -> ClientResult<()> {
    let realm_address = get_realm_address(&GOVERNANCE_PROGRAM, GOVERNANCE_REALM_DAO);
    let realm = match states.network.get_account(&realm_address).await? {
        None => bail!("failed to get realm account {realm_address}"),

        Some(acc) => match RealmV2::deserialize(&mut &acc.data[..]) {
            Ok(realm) => realm,
            Err(_) => bail!("failed to deserialize realm account {realm_address}"),
        },
    };

    let governances = states
        .network
        .get_program_accounts(
            &GOVERNANCE_PROGRAM,
            &[AccountFilter::Memcmp {
                bytes: vec![GovernanceAccountType::GovernanceV2 as u8],
                offset: 0,
            }],
        )
        .await?
        .into_iter()
        .filter_map(
            |(address, account)| match GovernanceV2::deserialize(&mut &account.data[..]) {
                Ok(governance) if governance.realm == realm_address => Some((address, governance)),
                Ok(_) => None,
                Err(err) => {
                    log::debug!(
                        "failed to deserialize governance account {address}: {}",
                        err
                    );
                    None
                }
            },
        )
        .collect::<HashMap<_, _>>();

    let proposals = states
        .network
        .get_program_accounts(
            &GOVERNANCE_PROGRAM,
            &[AccountFilter::Memcmp {
                bytes: vec![GovernanceAccountType::ProposalV2 as u8],
                offset: 0,
            }],
        )
        .await?
        .into_iter()
        .filter_map(
            |(address, account)| match ProposalV2::deserialize(&mut &account.data[..]) {
                Ok(proposal) if governances.contains_key(&proposal.governance) => {
                    Some((address, proposal))
                }
                Ok(_) => None,
                Err(err) => {
                    log::debug!("failed to deserialize proposal account {address}: {}", err);
                    None
                }
            },
        )
        .collect::<HashMap<_, _>>();

    let token_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM,
        &realm_address,
        &realm.community_mint,
        &states.wallet,
    );
    let vote_addrs = proposals
        .iter()
        .map(|(address, _)| {
            get_vote_record_address(&GOVERNANCE_PROGRAM, address, &token_owner_record)
        })
        .collect::<Vec<_>>();

    let vote_accounts = states.network.get_accounts_all(&vote_addrs).await?;
    let votes = vote_accounts
        .into_iter()
        .enumerate()
        .filter_map(|(i, acc)| {
            acc.and_then(|acc| match VoteRecordV2::deserialize(&mut &acc.data[..]) {
                Ok(vote) => Some((vote_addrs[i], vote)),
                Err(err) => {
                    log::debug!(
                        "failed to deserialize vote account {}: {}",
                        vote_addrs[i],
                        err
                    );
                    None
                }
            })
        })
        .collect();

    states.set(
        &realm_address,
        RealmInfo {
            governances,
            proposals,
            votes,
            state: realm,
        },
    );

    Ok(())
}

pub async fn sync_stake(states: &AccountStates) -> ClientResult<()> {
    let stake_pool = derive_stake_pool(DEFAULT_STAKE_SEED);
    let stake_account = derive_stake_account(&stake_pool, &states.wallet);

    match states
        .network
        .try_get_anchor_account::<StakePool>(&stake_pool)
        .await?
    {
        Some(data) => states.set(&stake_pool, data),
        None => log::warn!("stake pool {stake_pool} not found"),
    }

    match states
        .network
        .try_get_anchor_account::<StakeAccount>(&stake_account)
        .await?
    {
        Some(data) => states.set(&stake_account, data),
        None => log::debug!("stake account {stake_account} not found"),
    }

    Ok(())
}

use anchor_lang::prelude::*;

/// The governance action VoterWeight is evaluated for
#[derive(Clone, Debug, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub enum VoterWeightAction {
    /// Cast vote for a proposal. Target: Proposal
    CastVote,

    /// Comment a proposal. Target: Proposal
    CommentProposal,

    /// Create Governance within a realm. Target: Realm
    CreateGovernance,

    /// Create a proposal for a governance. Target: Governance
    CreateProposal,

    /// Signs off a proposal for a governance. Target: Proposal
    /// Note: SignOffProposal is not supported in the current version
    SignOffProposal,
}

/// VoterWeightRecord account
/// The account is used as an api interface to provide voting power to the governance program from external addin contracts
#[account]
#[derive(Debug, PartialEq, Eq)]
pub struct VoterWeightRecord {
    /// The Realm the VoterWeightRecord belongs to
    pub realm: Pubkey,

    /// Governing Token Mint the VoterWeightRecord is associated with
    /// Note: The addin can take deposits of any tokens and is not restricted to the community or council tokens only
    // The mint here is to link the record to either community or council mint of the realm
    pub governing_token_mint: Pubkey,

    /// The owner of the governing token and voter
    /// This is the actual owner (voter) and corresponds to TokenOwnerRecord.governing_token_owner
    pub owner: Pubkey,

    /// Voter's weight
    /// The weight of the voter provided by the addin for the given realm, governing_token_mint and governing_token_owner (voter)
    pub voter_weight: u64,

    /// The slot when the voting weight expires
    /// It should be set to None if the weight never expires
    /// If the voter weight decays with time, for example for time locked based weights, then the expiry must be set
    /// As a common pattern Revise instruction to update the weight should be invoked before governance instruction within the same transaction
    /// and the expiry set to the current slot to provide up to date weight
    pub voter_weight_expiry: Option<u64>,

    /// The governance action the voter's weight pertains to
    /// It allows to provided voter's weight specific to the particular action the weight is evaluated for
    /// When the action is provided then the governance program asserts the executing action is the same as specified by the addin
    pub weight_action: Option<VoterWeightAction>,

    /// The target the voter's weight  action pertains to
    /// It allows to provided voter's weight specific to the target the weight is evaluated for
    /// For example when addin supplies weight to vote on a particular proposal then it must specify the proposal as the action target
    /// When the target is provided then the governance program asserts the target is the same as specified by the addin
    pub weight_action_target: Option<Pubkey>,

    /// Reserved space for future versions
    pub reserved: [u8; 8],
}

/// MaxVoterWeightRecord account
/// The account is used as an api interface to provide max voting power to the governance program from external addin contracts
#[account]
#[derive(Debug, PartialEq, Eq)]
pub struct MaxVoterWeightRecord {
    /// The Realm the MaxVoterWeightRecord belongs to
    pub realm: Pubkey,

    /// Governing Token Mint the MaxVoterWeightRecord is associated with
    /// Note: The addin can take deposits of any tokens and is not restricted to the community or council tokens only
    // The mint here is to link the record to either community or council mint of the realm
    pub governing_token_mint: Pubkey,

    /// Max voter weight
    /// The max voter weight provided by the addin for the given realm and governing_token_mint
    pub max_voter_weight: u64,

    /// The slot when the max voting weight expires
    /// It should be set to None if the weight never expires
    /// If the max vote weight decays with time, for example for time locked based weights, then the expiry must be set
    /// As a pattern Revise instruction to update the max weight should be invoked before governance instruction within the same transaction
    /// and the expiry set to the current slot to provide up to date weight
    pub max_voter_weight_expiry: Option<u64>,

    /// Reserved space for future versions
    pub reserved: [u8; 8],
}

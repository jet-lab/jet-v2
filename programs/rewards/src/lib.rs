#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("JET777rQuPU8BatFbhp6irc1NAbozxTheBqNo25eLQP");

pub mod events;
pub mod instructions;
pub mod state;

pub use instructions::*;

pub mod seeds {
    use super::constant;

    #[constant]
    pub const AWARD: &[u8] = b"award";

    #[constant]
    pub const DISTRIBUTION: &[u8] = b"distribution";

    #[constant]
    pub const VAULT: &[u8] = b"vault";
}

#[program]
pub mod jet_rewards {
    use super::*;

    /// Initialize a new account to manage an airdrop, which can distribute
    /// tokens to a large set of accounts.
    pub fn airdrop_create(ctx: Context<AirdropCreate>, params: AirdropCreateParams) -> Result<()> {
        instructions::airdrop_create_handler(ctx, params)
    }

    /// Add recipients of an airdrop.
    ///
    /// Recipients have to be provided in sorted order based on the
    /// recipient's address.
    pub fn airdrop_add_recipients(
        ctx: Context<AirdropAddRecipients>,
        params: AirdropAddRecipientsParams,
    ) -> Result<()> {
        instructions::airdrop_add_recipients_handler(ctx, params)
    }

    /// Mark an airdrop account as final, preventing any further changes,
    /// and allowing recipients to claim their tokens.
    pub fn airdrop_finalize(ctx: Context<AirdropFinalize>) -> Result<()> {
        instructions::airdrop_finalize_handler(ctx)
    }

    /// Close and delete an airdrop account.
    pub fn airdrop_close(ctx: Context<AirdropClose>) -> Result<()> {
        instructions::airdrop_close_handler(ctx)
    }

    /// Claim of tokens from an airdrop as a recipient
    pub fn airdrop_claim(ctx: Context<AirdropClaim>) -> Result<()> {
        instructions::airdrop_claim_handler(ctx)
    }

    /// Initialize a new token distribution
    pub fn distribution_create(
        ctx: Context<DistributionCreate>,
        params: DistributionCreateParams,
    ) -> Result<()> {
        instructions::distribution_create_handler(ctx, params)
    }

    /// Release tokens from a distrubtion to the target
    pub fn distribution_release(ctx: Context<DistributionRelease>) -> Result<()> {
        instructions::distribution_release_handler(ctx)
    }

    /// Close a completed distribution
    pub fn distribution_close(ctx: Context<DistributionClose>) -> Result<()> {
        instructions::distribution_close_handler(ctx)
    }

    /// Create a new award, to vest tokens to a stake account over time
    pub fn award_create(ctx: Context<AwardCreate>, params: AwardCreateParams) -> Result<()> {
        instructions::award_create_handler(ctx, params)
    }

    /// Release vested tokens into the target stake account
    pub fn award_release(ctx: Context<AwardRelease>) -> Result<()> {
        instructions::award_release_handler(ctx)
    }

    /// Close a fully vested award
    pub fn award_close(ctx: Context<AwardClose>) -> Result<()> {
        instructions::award_close_handler(ctx)
    }

    /// Revoke an active award, reclaiming the unvested balance
    pub fn award_revoke(ctx: Context<AwardRevoke>) -> Result<()> {
        instructions::award_revoke_handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

mod error {
    use super::*;

    #[error_code(offset = 0)]
    pub enum ErrorCode {
        RecipientNotFound = 7000,
        AddOutOfOrder,
        AirdropFinal,
        AirdropInsufficientRewardBalance,
        AirdropExpired,
        AirdropNotFinal,
        RecipientsNotSorted,

        DistributionNotEnded,

        AwardNotFullyVested,
    }
}

pub use error::ErrorCode;

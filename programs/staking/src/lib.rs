#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use solana_program::pubkey;

declare_id!("JPLockxtkngHkaQT5AuRYow3HyUv5qWzmhwsCPd653n");

pub mod events;
mod instructions;
mod spl_addin;
pub mod state;

pub use instructions::PoolConfig;
use instructions::*;

pub mod seeds {
    use super::constant;

    #[constant]
    pub const COLLATERAL_MINT: &[u8] = b"collateral-mint";

    #[constant]
    pub const MAX_VOTE_WEIGHT_RECORD: &[u8] = b"max-vote-weight-record";

    #[constant]
    pub const VAULT: &[u8] = b"vault";

    #[constant]
    pub const VOTER_WEIGHT_RECORD: &[u8] = b"voter-weight-record";
}

#[program]
pub mod jet_staking {
    use super::*;

    /// Initialize a new pool that tokens can be staked to
    ///
    /// # Params
    ///
    /// * `seed` - A string to derive the pool address
    pub fn init_pool(ctx: Context<InitPool>, seed: String, config: PoolConfig) -> Result<()> {
        instructions::init_pool_handler(ctx, seed, config)
    }

    /// Initialize a new staking account
    ///
    /// The account created is tied to the owner that signed to create it.
    ///
    pub fn init_stake_account(ctx: Context<InitStakeAccount>) -> Result<()> {
        instructions::init_stake_account_handler(ctx)
    }

    /// Add tokens as stake to an account
    ///
    /// # Params
    ///
    /// * `amount` - The amount of tokens to transfer to the stake pool
    pub fn add_stake(ctx: Context<AddStake>, amount: Option<u64>) -> Result<()> {
        instructions::add_stake_handler(ctx, amount)
    }

    /// Unbond stake from an account, allowing it to be withdrawn
    pub fn unbond_stake(ctx: Context<UnbondStake>, seed: u32, amount: Option<u64>) -> Result<()> {
        instructions::unbond_stake_handler(ctx, seed, amount)
    }

    /// Cancel a previous request to unbond stake
    pub fn cancel_unbond(ctx: Context<CancelUnbond>) -> Result<()> {
        instructions::cancel_unbond_handler(ctx)
    }

    /// Withdraw stake that was previously unbonded
    pub fn withdraw_unbonded(ctx: Context<WithdrawUnbonded>) -> Result<()> {
        instructions::withdraw_unbonded_handler(ctx)
    }

    /// Withdraw stake from the pool by the authority
    pub fn withdraw_bonded(ctx: Context<WithdrawBonded>, amount: u64) -> Result<()> {
        instructions::withdraw_bonded_handler(ctx, amount)
    }

    /// Close out the stake account, return any rent
    pub fn close_stake_account(ctx: Context<CloseStakeAccount>) -> Result<()> {
        instructions::close_stake_account_handler(ctx)
    }
}

pub use error::ErrorCode;

mod error {
    use super::*;

    #[error_code(offset = 0)]
    #[derive(Eq, PartialEq)]
    pub enum ErrorCode {
        InsufficientStake = 7100,
        InvalidTokenOwnerRecord,
        OutstandingVotes,
        NotYetUnbonded,
        StakeRemaining,
        InvalidAmount,
    }
}

pub mod spl_governance {
    use super::declare_id;

    declare_id!("JPGov2SBA6f7XSJF5R4Si5jEJekGiyrwP2m7gSEqLUs");
}

#[derive(Copy, Clone)]
pub struct SplGovernance;

impl Id for SplGovernance {
    fn id() -> Pubkey {
        pubkey!("JPGov2SBA6f7XSJF5R4Si5jEJekGiyrwP2m7gSEqLUs")
    }
}

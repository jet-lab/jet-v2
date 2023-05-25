use std::io::Write;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{Mint, Token};

use crate::events::StakePoolCreated;
use crate::seeds;
use crate::spl_addin::MaxVoterWeightRecord;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PoolConfig {
    /// The time period for unbonding staked tokens from the pool.
    ///
    /// Unit is seconds.
    pub unbond_period: u64,

    /// The governance realm that the pool has voting power in
    pub governance_realm: Pubkey,
}

#[derive(Accounts)]
#[instruction(seed: String, config: PoolConfig)]
pub struct InitPool<'info> {
    /// The address paying to create this pool
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The address allowed to sign for changes to the pool,
    /// and management of the token balance.
    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// The mint for the tokens being staked into the pool.
    pub token_mint: Account<'info, Mint>,

    /// The new pool being created
    #[account(
        init,
        payer = payer,
        seeds = [seed.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<StakePool>(),
    )]
    pub stake_pool: Box<Account<'info, StakePool>>,

    /// The max voter weight
    #[account(init,
              seeds = [
                  config.governance_realm.as_ref(),
                  seeds::MAX_VOTE_WEIGHT_RECORD,
              ],
              bump,
              payer = payer,
              space = 8 + std::mem::size_of::<MaxVoterWeightRecord>())]
    pub max_voter_weight_record: Account<'info, MaxVoterWeightRecord>,

    /// The mint to issue derived collateral tokens
    #[account(init,
              seeds = [
                  seed.as_bytes(),
                  seeds::COLLATERAL_MINT,
              ],
              bump,
              payer = payer,
              mint::decimals = token_mint.decimals,
              mint::authority = stake_pool)]
    pub stake_collateral_mint: Account<'info, Mint>,

    /// The token account that stores the tokens staked into the pool.
    #[account(init,
              seeds = [
                  seed.as_bytes(),
                  seeds::VAULT,
              ],
              bump,
              payer = payer,
              token::mint = token_mint,
              token::authority = stake_pool)]
    pub stake_pool_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_pool_handler(ctx: Context<InitPool>, seed: String, config: PoolConfig) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let max_voter_weight_record = &mut ctx.accounts.max_voter_weight_record;

    stake_pool.authority = ctx.accounts.authority.key();
    stake_pool.token_mint = ctx.accounts.token_mint.key();
    stake_pool.stake_pool_vault = ctx.accounts.stake_pool_vault.key();
    stake_pool.max_voter_weight_record = max_voter_weight_record.key();
    stake_pool.governance_realm = config.governance_realm;

    stake_pool.bump_seed[0] = *ctx.bumps.get("stake_pool").unwrap();
    stake_pool.seed.as_mut().write_all(seed.as_bytes())?;
    stake_pool.seed_len = seed.len() as u8;

    stake_pool.unbond_period = config.unbond_period as i64;

    max_voter_weight_record.realm = config.governance_realm;
    max_voter_weight_record.governing_token_mint = stake_pool.token_mint;
    stake_pool.update_max_vote_weight_record(max_voter_weight_record);

    emit!(StakePoolCreated {
        stake_pool: stake_pool.key(),
        authority: ctx.accounts.authority.key(),
        seed,
        token_mint: stake_pool.token_mint,
        config,
        max_voter_weight: max_voter_weight_record.max_voter_weight,
    });

    Ok(())
}

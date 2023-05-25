use std::io::Write;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{events, seeds, state::*};

#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct AirdropCreateParams {
    /// The expiration time for the airdrop
    pub expire_at: i64,

    /// The stake pool that claimed rewards are deposited into.
    pub stake_pool: Pubkey,

    /// A description for this airdrop
    pub short_desc: String,

    /// A longer description for this airdrop
    pub long_desc: String,

    /// Airdrop settings
    pub flags: u64,
}

#[derive(Accounts)]
#[instruction(params: AirdropCreateParams)]
pub struct AirdropCreate<'info> {
    /// The account to store all the airdrop metadata
    #[account(zero)]
    pub airdrop: AccountLoader<'info, Airdrop>,

    /// The address that will have authority over the airdrop
    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// The account to store the tokens to be distributed
    /// as a reward via the airdrop
    #[account(init,
              seeds = [
                  airdrop.key().as_ref(),
                  seeds::VAULT,
              ],
              bump,
              payer = payer,
              token::mint = token_mint,
              token::authority = reward_vault)]
    pub reward_vault: Account<'info, TokenAccount>,

    /// The payer for rent charges
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The reward token's mint
    /// CHECK:
    pub token_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn airdrop_create_handler(
    ctx: Context<AirdropCreate>,
    params: AirdropCreateParams,
) -> Result<()> {
    let mut airdrop = ctx.accounts.airdrop.load_init()?;

    airdrop.address = ctx.accounts.airdrop.key();
    airdrop.authority = ctx.accounts.authority.key();
    airdrop.reward_vault = ctx.accounts.reward_vault.key();
    airdrop.vault_bump[0] = *ctx.bumps.get("reward_vault").unwrap();

    airdrop.expire_at = params.expire_at;
    airdrop.stake_pool = params.stake_pool;

    airdrop.flags = params.flags;

    airdrop
        .short_desc
        .as_mut()
        .write_all(params.short_desc.as_bytes())?;
    airdrop
        .long_desc
        .as_mut()
        .write_all(params.long_desc.as_bytes())?;

    emit!(events::AirdropCreated {
        airdrop: airdrop.address,
        authority: ctx.accounts.authority.key(),
        token_mint: ctx.accounts.token_mint.key(),
        params,
    });

    Ok(())
}

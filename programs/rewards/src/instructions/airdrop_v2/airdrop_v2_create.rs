use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{seeds, state::*};

#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct AirdropV2CreateParams {
    /// The seed for the airdrop
    pub seed: u64,

    /// The expiration time for the airdrop
    pub expire_at: i64,

    /// The stake pool that claimed rewards are deposited into.
    pub stake_pool: Pubkey,

    /// The address that will have initial authority over the airdrop
    pub authority: Pubkey,

    /// A description for this airdrop
    pub short_desc: String,
}

#[derive(Accounts)]
#[instruction(params: AirdropV2CreateParams)]
pub struct AirdropV2Create<'info> {
    /// The account to store all the airdrop metadata
    #[account(init,
              seeds = [
                  seeds::AIRDROP,
                  params.seed.to_le_bytes().as_ref(),
              ],
              bump,
              payer = payer,
              space = 8 + AirdropV2::MINIMUM_SIZE
    )]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    /// The account to store the tokens to be distributed
    /// as a reward via the airdrop
    #[account(init,
              seeds = [
                  seeds::VAULT,
                  airdrop.key().as_ref(),
              ],
              bump,
              payer = payer,
              token::mint = token_mint,
              token::authority = airdrop)]
    pub vault: Account<'info, TokenAccount>,

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

pub fn airdrop_v2_create_handler(
    ctx: Context<AirdropV2Create>,
    params: AirdropV2CreateParams,
) -> Result<()> {
    let create_params = AirdropCreateParams {
        authority: params.authority,
        expire_at: params.expire_at,
        stake_pool: params.stake_pool,
        short_desc: params.short_desc,
        seed: params.seed,
        bump_seed: *ctx.bumps.get("airdrop").unwrap(),
        vault: ctx.accounts.vault.key(),
    };

    let _ = AirdropV2::initialize(ctx.accounts.airdrop.as_ref(), &create_params)?;

    Ok(())
}

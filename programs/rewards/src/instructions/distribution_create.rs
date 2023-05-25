use std::io::Write;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{events, seeds, state::*};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DistributionCreateParams {
    /// The seed to create the address for the distribution
    pub seed: String,

    /// The authority allowed to manage the distribution
    pub authority: Pubkey,

    /// The token account to send the distributed tokens to
    pub target_account: Pubkey,

    /// The amount of tokens to be distributed
    pub amount: u64,

    /// Time distribution starts at
    pub begin_at: u64,

    /// Time distribution is completed at
    pub end_at: u64,
}

#[derive(Accounts)]
#[instruction(params: DistributionCreateParams)]
pub struct DistributionCreate<'info> {
    /// The account to store the distribution info
    #[account(
        init,
        payer = payer_rent,
        seeds = [
            seeds::DISTRIBUTION,
            params.seed.as_bytes()
        ],
        bump,
        space = 8 + Distribution::space(),
    )]
    pub distribution: Account<'info, Distribution>,

    /// The account to store the tokens to be distributed
    #[account(init,
              seeds = [
                  distribution.key().as_ref(),
                  seeds::VAULT,
              ],
              bump,
              payer = payer_rent,
              token::mint = token_mint,
              token::authority = distribution)]
    pub vault: Account<'info, TokenAccount>,

    /// The payer for rent charges
    #[account(mut)]
    pub payer_rent: Signer<'info>,

    /// The payer providing the tokens to be distributed
    pub payer_token_authority: Signer<'info>,

    /// The account to source the tokens to be distributed
    /// CHECK:
    #[account(mut)]
    pub payer_token_account: UncheckedAccount<'info>,

    /// The distribution token's mint
    /// CHECK:
    pub token_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> DistributionCreate<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.payer_token_account.to_account_info(),
                to: self.vault.to_account_info(),
                authority: self.payer_token_authority.to_account_info(),
            },
        )
    }
}

pub fn distribution_create_handler(
    ctx: Context<DistributionCreate>,
    params: DistributionCreateParams,
) -> Result<()> {
    let distribution = &mut ctx.accounts.distribution;

    distribution.address = distribution.key();
    distribution
        .seed
        .as_mut()
        .write_all(params.seed.as_bytes())?;
    distribution.seed_len = params.seed.len() as u8;
    distribution.bump_seed[0] = *ctx.bumps.get("distribution").unwrap();

    distribution.authority = params.authority;
    distribution.vault = ctx.accounts.vault.key();
    distribution.target_account = params.target_account;
    distribution.target_amount = params.amount;
    distribution.begin_at = params.begin_at;
    distribution.end_at = params.end_at;
    distribution.kind = DistributionKind::Linear;

    let distribution = &ctx.accounts.distribution;

    token::transfer(ctx.accounts.transfer_context(), params.amount)?;

    emit!(events::DistributionCreated {
        distribution: distribution.key(),
        authority: distribution.authority,
        token_mint: ctx.accounts.token_mint.key(),
        params,
        distribution_kind: distribution.kind,
    });

    Ok(())
}

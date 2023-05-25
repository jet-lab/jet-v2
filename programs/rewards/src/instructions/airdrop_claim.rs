use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use jet_staking::cpi::accounts::AddStake;
use jet_staking::program::JetStaking;

use crate::ErrorCode;
use crate::{events, state::*};

#[derive(Accounts)]
pub struct AirdropClaim<'info> {
    /// The airdrop to claim from
    #[account(mut,
              has_one = stake_pool,
              has_one = reward_vault)]
    pub airdrop: AccountLoader<'info, Airdrop>,

    /// The token account to claim the rewarded tokens from
    /// CHECK:
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,

    /// The address entitled to the airdrop, which must sign to claim
    pub recipient: Signer<'info>,

    /// The address to receive rent recovered from the claim account
    /// CHECK:
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    /// The stake pool to deposit stake into
    /// CHECK:
    #[account(mut)]
    pub stake_pool: AccountInfo<'info>,

    /// The stake pool token vault
    /// CHECK:
    #[account(mut)]
    pub stake_pool_vault: UncheckedAccount<'info>,

    /// The account to own the stake being deposited
    /// CHECK:
    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    /// The voter weight for the stake account
    /// CHECK:
    #[account(mut)]
    pub voter_weight_record: AccountInfo<'info>,

    /// The max voter weight
    /// CHECK:
    #[account(mut)]
    pub max_voter_weight_record: AccountInfo<'info>,

    pub staking_program: Program<'info, JetStaking>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AirdropClaim<'info> {
    fn add_stake_context(&self) -> CpiContext<'_, '_, '_, 'info, AddStake<'info>> {
        CpiContext::new(
            self.staking_program.to_account_info(),
            AddStake {
                stake_pool: self.stake_pool.to_account_info(),
                stake_pool_vault: self.stake_pool_vault.to_account_info(),
                stake_account: self.stake_account.to_account_info(),
                voter_weight_record: self.voter_weight_record.to_account_info(),
                max_voter_weight_record: self.max_voter_weight_record.to_account_info(),
                payer: self.reward_vault.to_account_info(),
                payer_token_account: self.reward_vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

pub fn airdrop_claim_handler(ctx: Context<AirdropClaim>) -> Result<()> {
    let mut airdrop = ctx.accounts.airdrop.load_mut()?;
    let clock = Clock::get()?;

    if airdrop.expire_at <= clock.unix_timestamp {
        msg!("this airdrop is expired");
        return Err(ErrorCode::AirdropExpired.into());
    }

    let claimed_amount = airdrop.claim(&ctx.accounts.recipient.key())?;

    jet_staking::cpi::add_stake(
        ctx.accounts
            .add_stake_context()
            .with_signer(&[&airdrop.signer_seeds()]),
        Some(claimed_amount),
    )?;

    emit!(events::AirdropClaimed {
        airdrop: airdrop.address,
        recipient: ctx.accounts.recipient.key(),
        claimed_amount,
        remaining_amount: airdrop.target_info().reward_total,

        vault_balance: ctx.accounts.reward_vault.amount,
    });

    Ok(())
}

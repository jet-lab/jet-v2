use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use jet_staking::cpi::accounts::AddStake;
use jet_staking::program::JetStaking;

use crate::ErrorCode;
use crate::{events, state::*};

#[derive(Accounts)]
pub struct AirdropV2Claim<'info> {
    /// The airdrop to claim from
    #[account(mut,
              has_one = stake_pool,
              has_one = vault)]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    /// The token account to claim the rewarded tokens from
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The address entitled to the airdrop, which must sign to claim
    pub recipient: Signer<'info>,

    /// The stake pool to deposit stake into
    #[account(mut)]
    pub stake_pool: AccountInfo<'info>,

    /// The stake pool token vault
    #[account(mut)]
    pub stake_pool_vault: UncheckedAccount<'info>,

    /// The account to own the stake being deposited
    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    /// The voter weight for the stake account
    #[account(mut)]
    pub voter_weight_record: AccountInfo<'info>,

    /// The max voter weight
    #[account(mut)]
    pub max_voter_weight_record: AccountInfo<'info>,

    pub staking_program: Program<'info, JetStaking>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AirdropV2Claim<'info> {
    fn add_stake_context(&self) -> CpiContext<'_, '_, '_, 'info, AddStake<'info>> {
        CpiContext::new(
            self.staking_program.to_account_info(),
            AddStake {
                stake_pool: self.stake_pool.to_account_info(),
                stake_pool_vault: self.stake_pool_vault.to_account_info(),
                stake_account: self.stake_account.to_account_info(),
                voter_weight_record: self.voter_weight_record.to_account_info(),
                max_voter_weight_record: self.max_voter_weight_record.to_account_info(),
                payer: self.airdrop.to_account_info(),
                payer_token_account: self.vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

pub fn airdrop_v2_claim_handler(ctx: Context<AirdropV2Claim>) -> Result<()> {
    let mut airdrop = AirdropV2::from_account(ctx.accounts.airdrop.as_ref())?;
    let clock = Clock::get()?;

    if airdrop.expire_at <= clock.unix_timestamp {
        msg!("this airdrop is expired");
        return Err(ErrorCode::AirdropExpired.into());
    }

    let claimed_amount = airdrop.claim(&ctx.accounts.recipient.key())?;
    let remaining_amount = airdrop.amount;
    //let signer = Seeds::new(&airdrop.signer_seeds());

    jet_staking::cpi::add_stake(
        ctx.accounts
            .add_stake_context()
            .with_signer(&[&airdrop.signer_seeds()]),
        Some(claimed_amount),
    )?;

    ctx.accounts.vault.reload()?;

    emit!(events::AirdropClaimed {
        airdrop: ctx.accounts.airdrop.key(),
        recipient: ctx.accounts.recipient.key(),
        claimed_amount,
        remaining_amount,

        vault_balance: ctx.accounts.vault.amount,
    });

    Ok(())
}

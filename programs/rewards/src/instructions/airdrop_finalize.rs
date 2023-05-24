use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};

use crate::{events, state::*};

#[derive(Accounts)]
pub struct AirdropFinalize<'info> {
    /// The airdrop to finalize
    #[account(mut,
              has_one = authority,
              has_one = reward_vault)]
    pub airdrop: AccountLoader<'info, Airdrop>,

    /// The token account holding the reward tokens to be distributed
    pub reward_vault: Account<'info, TokenAccount>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,
}

pub fn airdrop_finalize_handler(ctx: Context<AirdropFinalize>) -> Result<()> {
    let mut airdrop = ctx.accounts.airdrop.load_mut()?;
    let vault_balance = token::accessor::amount(&ctx.accounts.reward_vault.to_account_info())?;

    airdrop.finalize(vault_balance)?;

    let info = airdrop.target_info();
    emit!(events::AirdropFinalized {
        airdrop: airdrop.address,
        reward_total: info.reward_total,
        recipients_total: info.recipients_total,

        vault_balance: ctx.accounts.reward_vault.amount,
    });

    Ok(())
}

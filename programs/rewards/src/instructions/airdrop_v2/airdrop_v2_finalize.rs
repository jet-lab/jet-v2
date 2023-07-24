use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::{state::*, ErrorCode};

#[derive(Accounts)]
pub struct AirdropV2Finalize<'info> {
    /// The airdrop to finalize
    #[account(mut,
              has_one = authority,
              has_one = vault)]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    /// The token account holding the reward tokens to be distributed
    pub vault: Account<'info, TokenAccount>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,
}

pub fn airdrop_v2_finalize_handler(ctx: Context<AirdropV2Finalize>) -> Result<()> {
    let mut airdrop = AirdropV2::from_account(ctx.accounts.airdrop.as_ref())?;

    if ctx.accounts.vault.amount < airdrop.amount {
        return err!(ErrorCode::AirdropInsufficientRewardBalance);
    }

    airdrop.finalize();

    Ok(())
}

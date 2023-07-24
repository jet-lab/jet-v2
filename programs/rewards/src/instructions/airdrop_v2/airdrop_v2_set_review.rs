use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
pub struct AirdropV2SetReview<'info> {
    /// The airdrop to finalize
    #[account(mut, has_one = authority)]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,
}

pub fn airdrop_v2_set_review_handler(
    ctx: Context<AirdropV2SetReview>,
    reviewer: Pubkey,
) -> Result<()> {
    let mut airdrop = AirdropV2::from_account(ctx.accounts.airdrop.as_ref())?;

    airdrop.change_authority(reviewer);
    airdrop.finalize_recipients();

    Ok(())
}

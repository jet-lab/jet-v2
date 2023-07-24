use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction(recipients: Vec<AirdropRecipient>)]
pub struct AirdropV2AddRecipients<'info> {
    /// The airdrop to add to
    #[account(mut, 
              has_one = authority,
              realloc = airdrop.as_ref().data_len() + AirdropV2::extra_space_for_recipients(recipients.len()),
              realloc::payer = payer,
              realloc::zero = false,
    )]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,

    /// The payer for any additional rent
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn airdrop_v2_add_recipients_handler(
    ctx: Context<AirdropV2AddRecipients>,
    recipients: Vec<AirdropRecipient>,
) -> Result<()> {
    let mut airdrop = AirdropV2::from_account(ctx.accounts.airdrop.as_ref())?;
    airdrop.add_recipients(recipients)?;

    Ok(())
}

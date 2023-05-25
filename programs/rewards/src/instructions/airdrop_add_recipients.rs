use anchor_lang::prelude::*;

use crate::{events, state::*};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AirdropAddRecipientsParams {
    pub start_index: u64,
    pub recipients: Vec<AirdropRecipientParam>,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct AirdropRecipientParam {
    /// The amount to receive
    pub amount: u64,

    /// The address allowed to claim the airdrop amount
    pub recipient: Pubkey,
}

#[derive(Accounts)]
pub struct AirdropAddRecipients<'info> {
    /// The airdrop to add to
    #[account(mut, has_one = authority)]
    pub airdrop: AccountLoader<'info, Airdrop>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,
}

pub fn airdrop_add_recipients_handler(
    ctx: Context<AirdropAddRecipients>,
    params: AirdropAddRecipientsParams,
) -> Result<()> {
    let mut airdrop = ctx.accounts.airdrop.load_mut()?;

    let info = airdrop.target_info();
    let reward_0 = info.reward_total;

    airdrop.add_recipients(
        params.start_index,
        params.recipients.iter().map(|r| (r.recipient, r.amount)),
    )?;

    let info = airdrop.target_info();
    let reward_1 = info.reward_total;
    emit!(events::AirdropRecipientsAdded {
        airdrop: airdrop.address,
        reward_additional: reward_1 - reward_0,
        reward_total: reward_1,
        recipients_additional: params.recipients.len() as u64,
        recipients_total: info.recipients_total,
        recipients: params.recipients,
    });

    Ok(())
}

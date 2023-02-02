use anchor_lang::prelude::*;

use jet_airspace::state::GovernorId;

#[derive(Accounts)]
pub struct RecoverUninitialized<'info> {
    /// The authority that must sign to make this change
    pub governor: Signer<'info>,

    #[account(has_one = governor)]
    pub governor_id: Account<'info, GovernorId>,

    #[account(mut, owner = crate::ID)]
    pub uninitialized: AccountInfo<'info>,

    #[account(mut)]
    pub recipient: AccountInfo<'info>,
}

pub fn handler(ctx: Context<RecoverUninitialized>) -> Result<()> {
    let uninitialized = &ctx.accounts.uninitialized;
    let recipient = &ctx.accounts.recipient;

    assert_eq!(uninitialized.try_borrow_data()?[..8], [0u8; 8]);

    **recipient.try_borrow_mut_lamports()? += uninitialized.lamports();
    **uninitialized.try_borrow_mut_lamports()? = 0;

    Ok(())
}

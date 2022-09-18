use anchor_lang::prelude::*;

use crate::tickets::{
    events::TicketTransferred,
    state::{deserialize_ticket, TicketKind},
};

#[derive(Accounts)]
pub struct TransferTicketOwnership<'info> {
    /// The ticket to transfer, either a ClaimTicket or SplitTicket
    /// CHECK: handled by instruction logic
    #[account(mut)]
    pub ticket: UncheckedAccount<'info>,
    /// The current owner of the ticket
    pub current_owner: Signer<'info>,
}

pub fn handler(ctx: Context<TransferTicketOwnership>, new_owner: Pubkey) -> Result<()> {
    match deserialize_ticket(ctx.accounts.ticket.to_account_info())? {
        TicketKind::Claim(mut ticket) => {
            ticket.verify_owner(ctx.accounts.current_owner.key)?;
            ticket.owner = new_owner;
        }
        TicketKind::Split(mut ticket) => {
            ticket.verify_owner(ctx.accounts.current_owner.key)?;
            ticket.owner = new_owner;
        }
    };

    emit!(TicketTransferred {
        ticket: ctx.accounts.ticket.key(),
        previous_owner: ctx.accounts.current_owner.key(),
        new_owner,
    });

    Ok(())
}

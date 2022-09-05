use anchor_lang::prelude::*;

use crate::{
    orderbook::state::event_queue::{EventAdapterMetadata, EventQueue},
    BondsError,
};

#[derive(Accounts)]
pub struct PopAdapterEvents<'info> {
    /// AdapterEventQueue account owned by outside user or program
    #[account(
        mut,
        constraint = adapter_queue.load()?.owner == owner.key() @ BondsError::DoesNotOwnEventAdapter,
    )]
    pub adapter_queue: AccountLoader<'info, EventAdapterMetadata>,

    /// Signing authority over the AdapterEventQueue
    pub owner: Signer<'info>,
}

pub fn handler(ctx: Context<PopAdapterEvents>, num_events: u32) -> Result<()> {
    // checks are performed by anchor when loading accounts
    let mut queue = EventQueue::new_adapter(ctx.accounts.adapter_queue.to_account_info().data)?;
    queue.pop_events(num_events)
}

use anchor_lang::prelude::*;

use crate::{
    events::EventAdapterRegistered,
    orderbook::state::{event_queue::EventAdapterMetadata, user::OrderbookUser},
    seeds, BondsError,
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RegisterAdapterParams {
    /// Total capacity of the adapter
    /// Increases rent cost
    pub num_events: u32,
}

#[derive(Accounts)]
#[instruction(params: RegisterAdapterParams)]
pub struct RegisterAdapter<'info> {
    /// AdapterEventQueue account owned by outside user or program
    #[account(
        init,
        seeds = [
            seeds::EVENT_ADAPTER,
            bond_manager.key().as_ref(),
            owner.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = EventAdapterMetadata::space(params.num_events),
    )]
    pub adapter_queue: AccountLoader<'info, EventAdapterMetadata>,

    /// BondManager for this Adapter
    /// CHECK:
    pub bond_manager: UncheckedAccount<'info>,

    /// The OrderbookUser this adapter is registered to
    #[account(
        mut,
        has_one = user @ BondsError::UserDoesNotOwnAccount,
        has_one = bond_manager @ BondsError::WrongBondManager,
    )]
    pub orderbook_user: Account<'info, OrderbookUser>,

    /// The owner of the orderbook_user account
    pub user: Signer<'info>,

    /// Signing authority over this queue
    pub owner: Signer<'info>,

    /// Payer for the initialization rent of the queue
    #[account(mut)]
    pub payer: Signer<'info>,

    /// solana system program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterAdapter>, _params: RegisterAdapterParams) -> Result<()> {
    let mut adapter = ctx.accounts.adapter_queue.load_init()?;
    adapter.owner = ctx.accounts.owner.key();
    adapter.manager = ctx.accounts.bond_manager.key();

    ctx.accounts.orderbook_user.event_adapter = ctx.accounts.adapter_queue.key();

    emit!(EventAdapterRegistered {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user.key(),
        adapter: ctx.accounts.adapter_queue.key(),
    });

    Ok(())
}

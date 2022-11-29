use crate::{
    control::state::{BondManager, CrankAuthorization},
    orderbook::state::{EventQueue, OrderbookEvent},
    serialization, utils, BondsError,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ConsumeEventsLight<'info> {
    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = event_queue @ BondsError::WrongEventQueue,
    )]
    #[account(mut)]
    pub bond_manager: AccountLoader<'info, BondManager>,

    // aaob accounts
    /// CHECK: handled by aaob
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    #[account(
        has_one = crank @ BondsError::WrongCrankAuthority,
        constraint = crank_authorization.airspace == bond_manager.load()?.airspace @ BondsError::WrongAirspaceAuthorization
    )]
    pub crank_authorization: Account<'info, CrankAuthorization>,
    pub crank: Signer<'info>,

    /// The account paying rent for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    //
    // remaining_accounts: [EventAccounts],
}

#[account]
pub struct EventAccount {
    seqno: u64,
    user: Pubkey,
    event: OrderbookEvent,
}

fn handler(ctx: Context<ConsumeEventsLight>) -> Result<()> {
    // ctx.remaining_accounts
    let queue = EventQueue::deserialize_market(ctx.accounts.event_queue.to_account_info())?.iter();
    let mut manager = ctx.accounts.bond_manager.load_mut()?;
    for event_account in ctx.remaining_accounts {
        let event = queue.next();
        let account = serialization::init::<EventAccount>(
            event_account.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[],
        )?;
        utils::init! {
            account = EventAccount {
                seqno: manager.next_event,
                user: event_user(event),
                event: event,
            }
        }
        manager.next_event += 1;
    }

    Ok(())
}

fn event_user(event: &OrderbookEvent) -> Pubkey {
    match event {
        OrderbookEvent::Fill(fill) => fill.maker_info.fill_account,
        OrderbookEvent::Out(out) => out.info.out_account,
    }
}

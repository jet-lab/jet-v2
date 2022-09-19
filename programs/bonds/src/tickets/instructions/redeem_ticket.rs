use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use jet_proto_math::traits::SafeAdd;

use crate::{
    control::state::BondManager,
    tickets::{
        events::TicketRedeemed,
        state::{deserialize_ticket, TicketKind},
    },
    BondsError,
};

#[derive(Accounts)]
pub struct RedeemTicket<'info> {
    /// One of either `SplitTicket` or `ClaimTicket` for redemption
    /// CHECK: deserialization and checks handled in instruction
    #[account(mut)]
    pub ticket: UncheckedAccount<'info>,

    /// The account that owns the ticket
    #[account(mut)]
    pub ticket_holder: Signer<'info>,

    /// The token account designated to recieve the assets underlying the claim
    #[account(mut)]
    pub claimant_token_account: Account<'info, TokenAccount>,

    /// The BondManager responsible for the asset
    #[account(
        has_one = underlying_token_vault @ BondsError::WrongVault,
        constraint = !bond_manager.load()?.tickets_paused @ BondsError::TicketsPaused,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The vault stores the tokens of the underlying asset managed by the BondManager
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RedeemTicket>) -> Result<()> {
    let redeemable: u64;
    let maturation_timestamp: i64;

    match deserialize_ticket(ctx.accounts.ticket.to_account_info())? {
        TicketKind::Claim(ticket) => {
            ticket.verify_owner_manager(
                ctx.accounts.ticket_holder.key,
                &ctx.accounts.bond_manager.key(),
            )?;

            redeemable = ticket.redeemable;
            maturation_timestamp = ticket.maturation_timestamp;

            ticket.close(ctx.accounts.ticket_holder.to_account_info())?;
        }
        TicketKind::Split(ticket) => {
            ticket.verify_owner_manager(
                ctx.accounts.ticket_holder.key,
                &ctx.accounts.bond_manager.key(),
            )?;

            redeemable = ticket.principal.safe_add(ticket.interest)?;
            maturation_timestamp = ticket.maturation_timestamp;

            ticket.close(ctx.accounts.ticket_holder.to_account_info())?;
        }
    }

    let current_time = Clock::get()?.unix_timestamp;
    if current_time < maturation_timestamp {
        msg!(
            "Matures at slot: [{:?}]\nCurrent Slot: [{:?}]",
            maturation_timestamp,
            current_time
        );
        return err!(BondsError::ImmatureBond);
    }

    // transfer from the vault to the bond_holder
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                to: ctx.accounts.claimant_token_account.to_account_info(),
                from: ctx.accounts.underlying_token_vault.to_account_info(),
                authority: ctx.accounts.bond_manager.to_account_info(),
            },
        )
        .with_signer(&[&ctx.accounts.bond_manager.load()?.authority_seeds()]),
        redeemable,
    )?;
    emit!(TicketRedeemed {
        bond_manager: ctx.accounts.bond_manager.key(),
        ticket_holder: ctx.accounts.ticket_holder.key(),
        redeemed_value: redeemable,
        maturation_timestamp,
        redeemed_timestamp: current_time,
    });
    Ok(())
}

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

    /// The account that must sign to redeem the ticket
    #[account(mut)]
    pub authority: Signer<'info>,

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

impl<'info> RedeemTicket<'info> {
    pub fn redeem(&self, ticket_holder: Pubkey) -> Result<u64> {
        let redeemable: u64;
        let maturation_timestamp: i64;

        match deserialize_ticket(self.ticket.to_account_info())? {
            TicketKind::Claim(ticket) => {
                ticket.verify_owner_manager(&ticket_holder, &self.bond_manager.key())?;
                redeemable = ticket.redeemable;
                maturation_timestamp = ticket.maturation_timestamp;
                ticket.close(self.authority.to_account_info())?;
            }
            TicketKind::Split(ticket) => {
                ticket.verify_owner_manager(&ticket_holder, &self.bond_manager.key())?;
                redeemable = ticket.principal.safe_add(ticket.interest)?;
                maturation_timestamp = ticket.maturation_timestamp;
                ticket.close(self.authority.to_account_info())?;
            }
        }

        let current_time = Clock::get()?.unix_timestamp;
        if current_time < maturation_timestamp {
            msg!(
                "Matures at time: [{:?}]\nCurrent time: [{:?}]",
                maturation_timestamp,
                current_time
            );
            return err!(BondsError::ImmatureBond);
        }

        // transfer from the vault to the bond_holder
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.claimant_token_account.to_account_info(),
                    from: self.underlying_token_vault.to_account_info(),
                    authority: self.bond_manager.to_account_info(),
                },
            )
            .with_signer(&[&self.bond_manager.load()?.authority_seeds()]),
            redeemable,
        )?;
        emit!(TicketRedeemed {
            bond_manager: self.bond_manager.key(),
            ticket_holder,
            redeemed_value: redeemable,
            maturation_timestamp,
            redeemed_timestamp: current_time,
        });

        Ok(redeemable)
    }
}

pub fn handler(ctx: Context<RedeemTicket>) -> Result<()> {
    ctx.accounts.redeem(ctx.accounts.authority.key())?;

    Ok(())
}

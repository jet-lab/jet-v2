use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use jet_program_common::traits::SafeAdd;

use crate::{
    control::state::MarketManager,
    tickets::{
        events::TicketRedeemed,
        state::{deserialize_ticket, TicketKind},
    },
    ErrorCode,
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

    /// The token account designated to receive the assets underlying the claim
    #[account(mut)]
    pub claimant_token_account: Account<'info, TokenAccount>,

    /// The MarketManager responsible for the asset
    #[account(
        has_one = underlying_token_vault @ ErrorCode::WrongVault,
        constraint = !market_manager.load()?.tickets_paused @ ErrorCode::TicketsPaused,
    )]
    pub market_manager: AccountLoader<'info, MarketManager>,

    /// The vault stores the tokens of the underlying asset managed by the MarketManager
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
                ticket.verify_owner_manager(&ticket_holder, &self.market_manager.key())?;
                redeemable = ticket.redeemable;
                maturation_timestamp = ticket.maturation_timestamp;
                ticket.close(self.authority.to_account_info())?;
            }
            TicketKind::Split(ticket) => {
                ticket.verify_owner_manager(&ticket_holder, &self.market_manager.key())?;
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
            return err!(ErrorCode::ImmatureMarketTicket);
        }

        // transfer from the vault to the ticket_holder
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.claimant_token_account.to_account_info(),
                    from: self.underlying_token_vault.to_account_info(),
                    authority: self.market_manager.to_account_info(),
                },
            )
            .with_signer(&[&self.market_manager.load()?.authority_seeds()]),
            redeemable,
        )?;
        emit!(TicketRedeemed {
            market_manager: self.market_manager.key(),
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

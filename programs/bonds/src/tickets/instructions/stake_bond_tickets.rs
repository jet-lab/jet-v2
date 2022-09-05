use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use jet_proto_math::traits::SafeAdd;

use crate::{
    control::state::BondManager,
    seeds,
    tickets::{events::TicketsStaked, state::ClaimTicket},
    BondsError,
};

/// Params needed to stake bond tickets
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeBondTicketsParams {
    /// number of tickets to stake
    pub amount: u64,
    /// uniqueness seed to allow a user to have many `ClaimTicket`s
    pub ticket_seed: Vec<u8>,
}

/// An instruction to stake held bond tickets
///
/// Creates a [ClaimTicket] that is redeemable after the bond tenor has passed
#[derive(Accounts)]
#[instruction(params: StakeBondTicketsParams)]
pub struct StakeBondTickets<'info> {
    /// A struct used to track maturation and total claimable funds
    #[account(
        init,
        seeds = [
            seeds::CLAIM_TICKET,
            ticket_holder.key.as_ref(),
            bond_manager.key().as_ref(),
            params.ticket_seed.as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<ClaimTicket>(),
    )]
    pub claim_ticket: Account<'info, ClaimTicket>,

    /// The BondManager account tracks bonded assets of a particular duration
    #[account(
        mut,
        has_one = bond_ticket_mint @ BondsError::WrongTicketMint,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The owner of bond tickets that wishes to stake them for a redeemable ticket
    pub ticket_holder: Signer<'info>,

    /// The account tracking the ticket_holder's bond tickets
    #[account(mut)]
    pub bond_ticket_token_account: Box<Account<'info, TokenAccount>>,

    /// The mint for the bond tickets for this instruction
    /// A mint is a specific instance of the token program for both the underlying asset and the bond duration
    #[account(mut)]
    pub bond_ticket_mint: Box<Account<'info, Mint>>,

    /// The payer for account initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The global on-chain `TokenProgram` for account authority transfer.
    pub token_program: Program<'info, Token>,

    /// The global on-chain `SystemProgram` for program account initialization.
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeBondTickets>, params: StakeBondTicketsParams) -> Result<()> {
    let StakeBondTicketsParams { amount, .. } = params;

    // Burn lenders' bond tokens
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.bond_ticket_mint.to_account_info(),
                from: ctx.accounts.bond_ticket_token_account.to_account_info(),
                authority: ctx.accounts.ticket_holder.to_account_info(),
            },
        ),
        amount,
    )?;

    let manager = ctx.accounts.bond_manager.load()?;

    let redeemable = manager.convert_tickets(amount)?;

    // Mint a claimable ticket for their burned tokens
    *ctx.accounts.claim_ticket = ClaimTicket {
        owner: ctx.accounts.ticket_holder.key(),
        bond_manager: ctx.accounts.bond_manager.key(),
        maturation_timestamp: Clock::get()?.unix_timestamp.safe_add(manager.duration)?,
        redeemable,
    };

    emit!(TicketsStaked {
        bond_manager: ctx.accounts.bond_manager.key(),
        ticket_holder: ctx.accounts.ticket_holder.key(),
        amount: params.amount,
    });

    Ok(())
}

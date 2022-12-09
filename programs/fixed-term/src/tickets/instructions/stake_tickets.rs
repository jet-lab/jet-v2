use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use jet_program_common::traits::SafeAdd;

use crate::{
    control::state::Market,
    seeds,
    tickets::{events::TicketsStaked, state::ClaimTicket},
    FixedTermErrorCode,
};

/// Params needed to stake tickets
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeTicketsParams {
    /// number of tickets to stake
    pub amount: u64,
    /// uniqueness seed to allow a user to have many `ClaimTicket`s
    pub ticket_seed: Vec<u8>,
}

/// An instruction to stake held tickets
///
/// Creates a [ClaimTicket] that is redeemable after the market tenor has passed
#[derive(Accounts)]
#[instruction(params: StakeTicketsParams)]
pub struct StakeTickets<'info> {
    /// A struct used to track maturation and total claimable funds
    #[account(
        init,
        seeds = [
            seeds::CLAIM_TICKET,
            market.key().as_ref(),
            ticket_holder.key.as_ref(),
            params.ticket_seed.as_slice(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<ClaimTicket>(),
    )]
    pub claim_ticket: Account<'info, ClaimTicket>,

    /// The Market account tracks fixed term market assets of a particular tenor
    #[account(
        mut,
        has_one = ticket_mint @ FixedTermErrorCode::WrongTicketMint,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The owner of tickets that wishes to stake them for a redeemable ticket
    pub ticket_holder: Signer<'info>,

    /// The account tracking the ticket_holder's tickets
    #[account(mut)]
    pub ticket_token_account: Box<Account<'info, TokenAccount>>,

    /// The mint for the tickets for this instruction
    /// A mint is a specific instance of the token program for both the underlying asset and the market tenor
    #[account(mut)]
    pub ticket_mint: Box<Account<'info, Mint>>,

    /// The payer for account initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The global on-chain `TokenProgram` for account authority transfer.
    pub token_program: Program<'info, Token>,

    /// The global on-chain `SystemProgram` for program account initialization.
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeTickets>, params: StakeTicketsParams) -> Result<()> {
    let StakeTicketsParams { amount, .. } = params;

    // Burn lenders' ticket tokens
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                from: ctx.accounts.ticket_token_account.to_account_info(),
                authority: ctx.accounts.ticket_holder.to_account_info(),
            },
        ),
        amount,
    )?;

    // Mint a claimable ticket for their burned tokens
    *ctx.accounts.claim_ticket = ClaimTicket {
        owner: ctx.accounts.ticket_holder.key(),
        market: ctx.accounts.market.key(),
        maturation_timestamp: Clock::get()?
            .unix_timestamp
            .safe_add(ctx.accounts.market.load()?.lend_tenor)?,
        redeemable: amount,
    };

    emit!(TicketsStaked {
        market: ctx.accounts.market.key(),
        ticket_holder: ctx.accounts.ticket_holder.key(),
        amount: params.amount,
    });

    Ok(())
}

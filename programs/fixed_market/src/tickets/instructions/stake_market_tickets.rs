use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use jet_program_common::traits::SafeAdd;

use crate::{
    control::state::Market,
    seeds,
    tickets::{events::TicketsStaked, state::ClaimTicket},
    ErrorCode,
};

/// Params needed to stake market tickets
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeMarketTicketsParams {
    /// number of tickets to stake
    pub amount: u64,
    /// uniqueness seed to allow a user to have many `ClaimTicket`s
    pub ticket_seed: Vec<u8>,
}

/// An instruction to stake held market tickets
///
/// Creates a [ClaimTicket] that is redeemable after the market tenor has passed
#[derive(Accounts)]
#[instruction(params: StakeMarketTicketsParams)]
pub struct StakeMarketTickets<'info> {
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

    /// The Market account tracks fixed market assets of a particular tenor
    #[account(
        mut,
        has_one = market_ticket_mint @ ErrorCode::WrongTicketMint,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The owner of market tickets that wishes to stake them for a redeemable ticket
    pub ticket_holder: Signer<'info>,

    /// The account tracking the ticket_holder's market tickets
    #[account(mut)]
    pub market_ticket_token_account: Box<Account<'info, TokenAccount>>,

    /// The mint for the market tickets for this instruction
    /// A mint is a specific instance of the token program for both the underlying asset and the market tenor
    #[account(mut)]
    pub market_ticket_mint: Box<Account<'info, Mint>>,

    /// The payer for account initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The global on-chain `TokenProgram` for account authority transfer.
    pub token_program: Program<'info, Token>,

    /// The global on-chain `SystemProgram` for program account initialization.
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeMarketTickets>, params: StakeMarketTicketsParams) -> Result<()> {
    let StakeMarketTicketsParams { amount, .. } = params;

    // Burn lenders' market tokens
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.market_ticket_mint.to_account_info(),
                from: ctx.accounts.market_ticket_token_account.to_account_info(),
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

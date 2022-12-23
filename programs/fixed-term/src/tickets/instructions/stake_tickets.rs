use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};

use crate::{
    control::state::Market,
    events::TermDepositCreated,
    seeds,
    tickets::{events::TicketsStaked, state::TermDeposit},
    FixedTermErrorCode,
};

/// Params needed to stake tickets
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeTicketsParams {
    /// number of tickets to stake
    pub amount: u64,

    /// uniqueness seed to allow a user to have many deposits
    pub seed: Vec<u8>,
}

/// An instruction to stake held tickets to mark a deposit
///
/// Creates a [TermDeposit] that is redeemable after the market tenor has passed
#[derive(Accounts)]
#[instruction(params: StakeTicketsParams)]
pub struct StakeTickets<'info> {
    /// A struct used to track maturation and total claimable funds
    #[account(
        init,
        seeds = [
            seeds::TERM_DEPOSIT,
            market.key().as_ref(),
            ticket_holder.key.as_ref(),
            params.seed.as_slice(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<TermDeposit>(),
    )]
    pub deposit: Account<'info, TermDeposit>,

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

    let matures_at = Clock::get()?.unix_timestamp + ctx.accounts.market.load()?.lend_tenor;

    // Mint a deposit for their burned tokens
    *ctx.accounts.deposit = TermDeposit {
        matures_at,
        sequence_number: 0,
        owner: ctx.accounts.ticket_holder.key(),
        market: ctx.accounts.market.key(),
        amount: params.amount,
        principal: params.amount,
    };

    emit!(TicketsStaked {
        market: ctx.accounts.market.key(),
        ticket_holder: ctx.accounts.ticket_holder.key(),
        amount: params.amount,
    });
    emit!(TermDepositCreated {
        term_deposit: ctx.accounts.deposit.key(),
        authority: ctx.accounts.ticket_holder.key(),
        order_tag: None,
        sequence_number: ctx.accounts.deposit.sequence_number,
        market: ctx.accounts.deposit.market,
        maturation_timestamp: matures_at,
        principal: params.amount,
        amount: params.amount,
    });

    Ok(())
}

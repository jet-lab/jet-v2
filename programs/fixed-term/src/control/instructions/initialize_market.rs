use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::{events::MarketInitialized, state::Market},
    seeds,
    utils::init,
};

/// Parameters for the initialization of the [Market]
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMarketParams {
    /// Tag information for the `Market` account
    pub version_tag: u64,
    /// This seed allows the creation of many separate ticket managers tracking different
    /// parameters, such as staking tenor
    pub seed: [u8; 32],
    /// Length of time before a borrow is marked as due, in seconds
    pub borrow_tenor: u64,
    /// Length of time before a claim is marked as mature, in seconds
    pub lend_tenor: u64,
    /// assessed on borrows. scaled by origination_fee::FEE_UNIT
    pub origination_fee: u64,
}

/// Initialize a [Market]
/// The `Market` acts as a sort of market header. Responsible for coordination and authorization of the accounts
/// utilized and interacting with the program
#[derive(Accounts)]
#[instruction(params: InitializeMarketParams)]
pub struct InitializeMarket<'info> {
    /// The `Market` manages asset tokens for a particular tenor
    #[account(
        init,
        seeds = [
            seeds::MARKET,
            airspace.key().as_ref(),
            underlying_token_mint.key().as_ref(),
            &params.seed,
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Market>(),
    )]
    pub market: AccountLoader<'info, Market>,

    /// The vault for storing the token underlying the tickets
    #[account(
        init,
        seeds = [
            seeds::UNDERLYING_TOKEN_VAULT,
            market.key().as_ref()
        ],
        bump,
        payer = payer,
        token::mint = underlying_token_mint,
        token::authority = market,
    )]
    pub underlying_token_vault: Box<Account<'info, TokenAccount>>,

    /// The mint for the assets underlying the tickets
    pub underlying_token_mint: Box<Account<'info, Mint>>,

    /// The minting account for the tickets
    #[account(
        init,
        seeds = [
            seeds::TICKET_MINT,
            market.key().as_ref()
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market,
        mint::freeze_authority = market,
    )]
    pub ticket_mint: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            market.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market,
        mint::freeze_authority = market,
    )]
    pub claims: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::TICKET_COLLATERAL_NOTES,
            market.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market,
        mint::freeze_authority = market,
    )]
    pub collateral: Box<Account<'info, Mint>>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ FixedTermErrorCode::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,

    /// The oracle for the underlying asset price
    /// CHECK: determined by caller
    pub underlying_oracle: AccountInfo<'info>,

    /// The oracle for the ticket price
    /// CHECK: determined by caller
    pub ticket_oracle: AccountInfo<'info>,

    /// The account where fees are allowed to be withdrawn
    #[account(token::mint = underlying_token_mint)]
    pub fee_destination: Box<Account<'info, TokenAccount>>,

    /// The account paying rent for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,

    /// SPL token program
    pub token_program: Program<'info, Token>,

    /// Solana system program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeMarket>, params: InitializeMarketParams) -> Result<()> {
    let market = &mut ctx.accounts.market.load_init()?;
    init! {
        market = Market {
            version_tag: params.version_tag,
            airspace: ctx.accounts.airspace.key(),
            underlying_token_mint: ctx.accounts.underlying_token_mint.key(),
            underlying_token_vault: ctx.accounts.underlying_token_vault.key(),
            ticket_mint: ctx.accounts.ticket_mint.key(),
            claims_mint: ctx.accounts.claims.key(),
            ticket_collateral_mint: ctx.accounts.collateral.key(),
            seed: params.seed,
            bump: [*ctx.bumps.get("market").unwrap()],
            orderbook_paused: false,
            tickets_paused: false,
            borrow_tenor: params.borrow_tenor,
            lend_tenor: params.lend_tenor,
            underlying_oracle: ctx.accounts.underlying_oracle.key(),
            ticket_oracle: ctx.accounts.ticket_oracle.key(),
            fee_destination: ctx.accounts.fee_destination.key(),
            origination_fee: params.origination_fee,
        } ignoring {
            orderbook_market_state,
            event_queue,
            asks,
            bids,
            nonce,
            collected_fees,
            _reserved,
        }
    }
    emit!(MarketInitialized {
        version: market.version_tag,
        address: ctx.accounts.market.key(),
        underlying_token_mint: market.underlying_token_mint,
        borrow_tenor: market.borrow_tenor,
        lend_tenor: market.lend_tenor,
        airspace: market.airspace,
        underlying_oracle: market.underlying_oracle,
        ticket_oracle: market.ticket_oracle,
    });

    Ok(())
}

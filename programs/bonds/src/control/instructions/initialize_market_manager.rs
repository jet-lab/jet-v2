use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::{events::MarketManagerInitialized, state::MarketManager},
    seeds,
    utils::init,
};

/// Parameters for the initialization of the [MarketManager]
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeMarketManagerParams {
    /// Tag information for the `MarketManager` account
    pub version_tag: u64,
    /// This seed allows the creation of many separate ticket managers tracking different
    /// parameters, such as staking tenor
    pub seed: [u8; 32],
    /// Length of time before a borrow is marked as due, in seconds
    pub borrow_tenor: i64,
    /// Length of time before a claim is marked as mature, in seconds
    pub lend_tenor: i64,
    /// assessed on borrows. scaled by origination_fee::FEE_UNIT
    pub origination_fee: u64,
}

/// Initialize a [MarketManager]
/// The `MarketManager` acts as a sort of market header. Responsible for coordination and authorization of the accounts
/// utilized and interacting with the program
#[derive(Accounts)]
#[instruction(params: InitializeMarketManagerParams)]
pub struct InitializeMarketManager<'info> {
    /// The `MarketManager` manages asset tokens for a particular tenor
    #[account(
        init,
        seeds = [
            seeds::MARKET_MANAGER,
            airspace.key().as_ref(),
            underlying_token_mint.key().as_ref(),
            &params.seed,
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<MarketManager>(),
    )]
    pub market_manager: AccountLoader<'info, MarketManager>,

    /// The vault for storing the token underlying the market tickets
    #[account(
        init,
        seeds = [
            seeds::UNDERLYING_TOKEN_VAULT,
            market_manager.key().as_ref()
        ],
        bump,
        payer = payer,
        token::mint = underlying_token_mint,
        token::authority = market_manager,
    )]
    pub underlying_token_vault: Box<Account<'info, TokenAccount>>,

    /// The mint for the assets underlying the market tickets
    pub underlying_token_mint: Box<Account<'info, Mint>>,

    /// The minting account for the market tickets
    #[account(
        init,
        seeds = [
            seeds::MARKET_TICKET_MINT,
            market_manager.key().as_ref()
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market_manager,
        mint::freeze_authority = market_manager,
    )]
    pub market_ticket_mint: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            market_manager.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market_manager,
        mint::freeze_authority = market_manager,
    )]
    pub claims: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::COLLATERAL_NOTES,
            market_manager.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = market_manager,
        mint::freeze_authority = market_manager,
    )]
    pub collateral: Box<Account<'info, Mint>>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ ErrorCode::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,

    /// The oracle for the underlying asset price
    /// CHECK: determined by caller
    pub underlying_oracle: AccountInfo<'info>,

    /// The oracle for the market ticket price
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

pub fn handler(
    ctx: Context<InitializeMarketManager>,
    params: InitializeMarketManagerParams,
) -> Result<()> {
    let manager = &mut ctx.accounts.market_manager.load_init()?;
    init! {
        manager = MarketManager {
            version_tag: params.version_tag,
            airspace: ctx.accounts.airspace.key(),
            underlying_token_mint: ctx.accounts.underlying_token_mint.key(),
            underlying_token_vault: ctx.accounts.underlying_token_vault.key(),
            market_ticket_mint: ctx.accounts.market_ticket_mint.key(),
            claims_mint: ctx.accounts.claims.key(),
            collateral_mint: ctx.accounts.collateral.key(),
            seed: params.seed,
            bump: [*ctx.bumps.get("market_manager").unwrap()],
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
    emit!(MarketManagerInitialized {
        version: manager.version_tag,
        address: ctx.accounts.market_manager.key(),
        underlying_token_mint: manager.underlying_token_mint,
        borrow_tenor: manager.borrow_tenor,
        lend_tenor: manager.lend_tenor,
        airspace: manager.airspace,
        underlying_oracle: manager.underlying_oracle,
        ticket_oracle: manager.ticket_oracle,
    });

    Ok(())
}

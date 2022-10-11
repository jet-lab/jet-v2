use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::{events::BondManagerInitialized, state::BondManager},
    seeds,
    utils::init,
};

/// Parameters for the initialization of the [BondManager]
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeBondManagerParams {
    /// Tag information for the `BondManager` account
    pub version_tag: u64,
    /// This seed allows the creation of many separate ticket managers tracking different
    /// parameters, such as staking duration
    pub seed: [u8; 32],
    /// Units added to the initial stake timestamp to determine claim maturity
    pub duration: i64,
}

/// Initialize a [BondManager]
/// The `BondManager` acts as a sort of market header. Responsible for coordination and authorization of the accounts
/// utilized and interacting with the program
#[derive(Accounts)]
#[instruction(params: InitializeBondManagerParams)]
pub struct InitializeBondManager<'info> {
    /// The `BondManager` manages asset tokens for a particular bond duration
    #[account(
        init,
        seeds = [
            seeds::BOND_MANAGER,
            airspace.key().as_ref(),
            underlying_token_mint.key().as_ref(),
            &params.seed,
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<BondManager>(),
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The vault for storing the token underlying the bond tickets
    #[account(
        init,
        seeds = [
            seeds::UNDERLYING_TOKEN_VAULT,
            bond_manager.key().as_ref()
        ],
        bump,
        payer = payer,
        token::mint = underlying_token_mint,
        token::authority = bond_manager,
    )]
    pub underlying_token_vault: Box<Account<'info, TokenAccount>>,

    /// The mint for the assets underlying the bond tickets
    pub underlying_token_mint: Box<Account<'info, Mint>>,

    /// The minting account for the bond tickets
    #[account(
        init,
        seeds = [
            seeds::BOND_TICKET_MINT,
            bond_manager.key().as_ref()
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = bond_manager,
        mint::freeze_authority = bond_manager,
    )]
    pub bond_ticket_mint: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::CLAIM_NOTES,
            bond_manager.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = bond_manager,
        mint::freeze_authority = bond_manager,
    )]
    pub claims: Box<Account<'info, Mint>>,

    /// Mints tokens to a margin account to represent debt that must be collateralized
    #[account(init,
        seeds = [
            seeds::COLLATERAL_NOTES,
            bond_manager.key().as_ref(),
        ],
        bump,
        payer = payer,
        mint::decimals = underlying_token_mint.decimals,
        mint::authority = bond_manager,
        mint::freeze_authority = bond_manager,
    )]
    pub collateral: Box<Account<'info, Mint>>,

    /// The authority that must sign to make this change
    pub authority: Signer<'info>,

    /// The airspace being modified
    // #[account(has_one = authority @ BondsError::WrongAirspaceAuthorization)] fixme airspace
    pub airspace: AccountInfo<'info>,

    /// The oracle for the underlying asset price
    /// CHECK: determined by caller
    pub underlying_oracle: AccountInfo<'info>,

    /// The oracle for the bond ticket price
    /// CHECK: determined by caller
    pub ticket_oracle: AccountInfo<'info>,

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
    ctx: Context<InitializeBondManager>,
    params: InitializeBondManagerParams,
) -> Result<()> {
    let manager = &mut ctx.accounts.bond_manager.load_init()?;
    init! {
        manager = BondManager {
            version_tag: params.version_tag,
            airspace: ctx.accounts.airspace.key(),
            underlying_token_mint: ctx.accounts.underlying_token_mint.key(),
            underlying_token_vault: ctx.accounts.underlying_token_vault.key(),
            bond_ticket_mint: ctx.accounts.bond_ticket_mint.key(),
            claims_mint: ctx.accounts.claims.key(),
            collateral_mint: ctx.accounts.collateral.key(),
            seed: params.seed,
            bump: [*ctx.bumps.get("bond_manager").unwrap()],
            orderbook_paused: false,
            tickets_paused: false,
            duration: params.duration,
            underlying_oracle: ctx.accounts.underlying_oracle.key(),
            ticket_oracle: ctx.accounts.ticket_oracle.key(),
        } ignoring {
            orderbook_market_state,
            event_queue,
            asks,
            bids,
            nonce,
            _reserved,
        }
    }
    emit!(BondManagerInitialized {
        version: manager.version_tag,
        address: ctx.accounts.bond_manager.key(),
        underlying_token_mint: manager.underlying_token_mint,
        duration: manager.duration,
        airspace: manager.airspace,
        underlying_oracle: manager.underlying_oracle,
        ticket_oracle: manager.ticket_oracle,
    });

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::{events::BondManagerInitialized, state::BondManager},
    seeds,
};

/// Parameters for the initialization of the [BondManager]
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeBondManagerParams {
    /// Tag information for the `BondManager` account
    pub version_tag: u64,
    /// This seed allows the creation of many separate ticket managers tracking different
    /// parameters, such as staking duration
    pub seed: u64,
    /// Units added to the initial stake timestamp to determine claim maturity
    pub duration: i64,
    /// The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`
    pub conversion_factor: i8,
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
            underlying_token_mint.key().as_ref(),
            params.seed.to_le_bytes().as_ref(),
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
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The mint for the assets underlying the bond tickets
    pub underlying_token_mint: Account<'info, Mint>,

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
    pub bond_ticket_mint: Account<'info, Mint>,

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
    pub claims: Account<'info, Mint>,

    /// The controlling signer for this program
    pub program_authority: Signer<'info>,

    /// The oracle for the underlying asset price
    pub oracle: AccountInfo<'info>,

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

    manager.version_tag = params.version_tag;
    manager.program_authority = ctx.accounts.program_authority.key();
    manager.underlying_token_mint = ctx.accounts.underlying_token_mint.key();
    manager.underlying_token_vault = ctx.accounts.underlying_token_vault.key();
    manager.bond_ticket_mint = ctx.accounts.bond_ticket_mint.key();
    manager.claims_mint = ctx.accounts.claims.key();
    manager.seed = params.seed.to_le_bytes();
    manager.bump = [*ctx.bumps.get("bond_manager").unwrap()];
    manager.conversion_factor = params.conversion_factor;
    manager.duration = params.duration;
    manager.oracle = ctx.accounts.oracle.key();

    emit!(BondManagerInitialized {
        version: manager.version_tag,
        address: ctx.accounts.bond_manager.key(),
        underlying_token: manager.underlying_token_mint,
        duration: manager.duration,
    });

    Ok(())
}

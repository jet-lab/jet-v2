#![allow(clippy::single_component_path_imports)]
use anchor_lang::prelude::*;
use anchor_spl::token::{burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount};
use jet_margin::{write_adapter_result, AdapterResult};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const SIGNER_SEED: [&[u8]; 0] = [];

pub fn signer() -> (Pubkey, u8) {
    Pubkey::find_program_address(&SIGNER_SEED, &crate::id())
}

#[program]
pub mod mock_adapter {
    use super::*;

    pub fn init_mint(_ctx: Context<InitMint>, _index: u8) -> Result<()> {
        Ok(())
    }

    pub fn mint_tokens(
        ctx: Context<MintAction>,
        amount: u64,
        result: Option<AdapterResult>,
    ) -> Result<()> {
        mint_ix!(ctx.accounts, amount, result)
    }

    pub fn burn_tokens(
        ctx: Context<MintAction>,
        amount: u64,
        result: Option<AdapterResult>,
    ) -> Result<()> {
        burn_ix!(ctx.accounts, amount, result)
    }

    pub fn mint_signed(
        ctx: Context<MintActionSigned>,
        amount: u64,
        result: Option<AdapterResult>,
    ) -> Result<()> {
        mint_ix!(ctx.accounts.action, amount, result)
    }

    pub fn burn_signed(
        ctx: Context<MintActionSigned>,
        amount: u64,
        result: Option<AdapterResult>,
    ) -> Result<()> {
        burn_ix!(ctx.accounts.action, amount, result)
    }

    pub fn noop(_ctx: Context<NoAccounts>, result: Option<AdapterResult>) -> Result<()> {
        match result {
            Some(result) => write_adapter_result(&result),
            None => Ok(()),
        }
    }
}

#[derive(Accounts)]
pub struct NoAccounts {}

#[derive(Accounts)]
pub struct MintAction<'info> {
    mint: Account<'info, Mint>,
    token_account: Account<'info, TokenAccount>,
    authority: AccountInfo<'info>,
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintActionSigned<'info> {
    owner: Signer<'info>,
    action: MintAction<'info>,
}

#[derive(Accounts)]
#[instruction(index: u8)]
pub struct InitMint<'info> {
    #[account(init,
        seeds = [&[index]],
        bump,
        mint::decimals = 0,
        mint::authority = authority,
        payer = payer)]
    mint: Account<'info, Mint>,

    authority: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, Token>,
    #[account(mut)]
    payer: Signer<'info>,
    rent: Sysvar<'info, Rent>,
}

macro_rules! mint_ix {
    ($accounts:expr, $amount:ident, $result:ident) => {{
        mint_to(
            CpiContext::new(
                $accounts.token_program.to_account_info(),
                MintTo {
                    mint: $accounts.mint.to_account_info(),
                    to: $accounts.token_account.to_account_info(),
                    authority: $accounts.authority.to_account_info(),
                },
            )
            .with_signer(&[&SIGNER_SEED]),
            $amount,
        )?;

        match $result {
            Some($result) => write_adapter_result(&$result),
            None => Ok(()),
        }
    }};
}
use mint_ix;

macro_rules! burn_ix {
    ($accounts:expr, $amount:ident, $result:ident) => {{
        burn(
            CpiContext::new(
                $accounts.token_program.to_account_info(),
                Burn {
                    mint: $accounts.mint.to_account_info(),
                    to: $accounts.token_account.to_account_info(),
                    authority: $accounts.authority.to_account_info(),
                },
            )
            .with_signer(&[&SIGNER_SEED]),
            $amount,
        )?;

        match $result {
            Some($result) => write_adapter_result(&$result),
            None => Ok(()),
        }
    }};
}
use burn_ix;

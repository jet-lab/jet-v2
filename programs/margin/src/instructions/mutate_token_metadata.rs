use anchor_lang::prelude::*;

use crate::{control::is_market_authority, PositionKind, TokenMeta};

#[derive(Accounts)]
pub struct MutateToken<'info> {
    #[account(mut,
		seeds = [
			b"token-metadata",
			other.token_mint.key().as_ref()
		],
		bump,
  	)]
    pub metadata: Account<'info, TokenMeta>,
    pub other: PositionTokenAccounts<'info>,
}

/// Requester is required. Otherwise pass in the system program to avoid changing the field
#[derive(Accounts)]
pub struct PositionTokenAccounts<'info> {
    #[account(mut, constraint = is_market_authority(&requester))]
    pub requester: Signer<'info>,

    pub token_mint: AccountInfo<'info>,

    /// The program that is allowed to:
    /// - set the price
    /// - register/close claim or adapter collateral positions
    pub adapter_program: AccountInfo<'info>,

    /// set iff margin is the adapter program
    pub pyth_price: AccountInfo<'info>,

    /// set iff margin is the adapter program
    pub pyth_product: AccountInfo<'info>,

    /// optional
    pub underlying_mint: AccountInfo<'info>,
}

pub fn mutate_token_handler(
    ctx: Context<MutateToken>,
    params: Option<PositionParams>,
) -> Result<()> {
    mutate_token_impl(&ctx.accounts.other, &mut ctx.accounts.metadata, params)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PositionParams {
    pub position_kind: PositionKind,
    pub value_modifier: u16,
    pub max_staleness: u64,
}

pub fn mutate_token_impl(
    accounts: &PositionTokenAccounts,
    metadata: &mut TokenMeta,
    params: Option<PositionParams>,
) -> Result<()> {
    let self_managed = accounts.adapter_program.key == &crate::ID;
    assert_eq!(self_managed, accounts.pyth_price.key != &Pubkey::default());
    assert_eq!(
        self_managed,
        accounts.pyth_product.key != &Pubkey::default()
    );

    if_not_default! {
        metadata = {
            underlying_mint: accounts.underlying_mint.key(),
            adapter_program: accounts.adapter_program.key(),
            pyth_price: accounts.pyth_price.key(),
            pyth_product: accounts.pyth_product.key(),
        }
    }

    if let Some(params) = params {
        metadata.position_kind = params.position_kind;
        metadata.value_modifier = params.value_modifier;
        metadata.max_staleness = params.max_staleness;
    }

    Ok(())
}

macro_rules! if_not_default {
    ($item:ident = {
        $($field:ident: $value:expr,)*
    }) => {
        $(if $value != Default::default() {
            $item.$field = $value;
        })*
    };
}
use if_not_default;

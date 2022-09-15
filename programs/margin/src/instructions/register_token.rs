use anchor_lang::prelude::*;

use crate::TokenMeta;

use super::mutate_token_metadata::*;

#[derive(Accounts)]
pub struct RegisterToken<'info> {
    #[account(init,
		seeds = [
			b"token-metadata",
			other.token_mint.key().as_ref()
		],
		bump,
		payer = other.requester,
		space = 8 + std::mem::size_of::<TokenMeta>(),
  	)]
    metadata: Account<'info, TokenMeta>,
    other: PositionTokenAccounts<'info>,
    system_program: Program<'info, System>,
}

pub fn register_token_handler(
    ctx: Context<RegisterToken>,
    params: Option<PositionParams>,
) -> Result<()> {
    ctx.accounts.metadata.token_mint = ctx.accounts.other.token_mint.key();
    mutate_token_impl(&ctx.accounts.other, &mut ctx.accounts.metadata, params)
}

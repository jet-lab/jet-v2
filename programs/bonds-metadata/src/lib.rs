use anchor_lang::prelude::*;

#[cfg(not(no_default_id))]
declare_id!("C8GWmni61jTvtdon55LJ5zkVGzyJuv5Mkq41YVaeyhGQ");

mod authority {
    use super::*;

    declare_id!("94YFSx1UGMmL2TN87yxKU6srzVqBq4zgfUm9b6pem4aS");
}

#[derive(Accounts)]
pub struct AuthorizeCrankSigner<'info> {
    /// The crank signer pubkey
    /// CHECK: Provided by caller
    pub crank_signer: AccountInfo<'info>,

    /// The account containing the metadata for the key
    /// CHECK: Provided by caller
    #[account(
        init,
        seeds = [
            crate::seeds::CRANK_SIGNER,
            crank_signer.key.as_ref()
        ],
        bump,
        space = std::mem::size_of::<CrankMetadata>() + 8,
        payer = payer
    )]
    pub metadata_account: Account<'info, CrankMetadata>,

    /// The authority that must sign to make this change
    #[cfg_attr(not(feature = "devnet"), account(address = authority::ID))]
    pub authority: Signer<'info>,

    /// The address paying the rent for the account
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeCrankSigner<'info> {
    /// The account containing the metadata for the key
    /// CHECK: Provided by caller
    #[account(
        mut,
        close = receiver
    )]
    pub metadata_account: Account<'info, CrankMetadata>,

    /// The authority that must sign to make this change
    #[cfg_attr(not(feature = "devnet"), account(address = authority::ID))]
    pub authority: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,
}

#[program]
pub mod jet_bonds_metadata {
    pub use super::*;

    pub fn authorize_crank_signer(ctx: Context<AuthorizeCrankSigner>) -> Result<()> {
        ctx.accounts.metadata_account.crank_signer = ctx.accounts.crank_signer.key();
        Ok(())
    }

    pub fn revoke_crank_signer(_ctx: Context<RevokeCrankSigner>) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct CrankMetadata {
    pub crank_signer: Pubkey,
}

pub mod seeds {
    use anchor_lang::prelude::constant;

    #[constant]
    pub const CRANK_SIGNER: &[u8] = b"crank_signer";
}

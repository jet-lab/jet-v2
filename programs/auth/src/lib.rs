#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
#[cfg(feature = "cli")]
use serde::ser::{Serialize, SerializeStruct, Serializer};

mod events;
use events::*;

declare_id!("JPALXR88jy2fG3miuu4n3o8Jef4K2Cgc3Uypr3Y8RNX");

/// Hardcoded address of the authority that can authenticate users
mod authority {
    use super::*;

    // The public key of the keypair authority in the Jet Protocol API
    declare_id!("23JJHdYjPL6uPAapH3JjaMgHmZUN29aAf9uSNdgk4TGC");
}

#[account]
#[derive(Debug)]
pub struct UserAuthentication {
    /// The relevant user address
    pub owner: Pubkey,

    /// Whether or not the authentication workflow for the user has
    /// already been completed.
    pub complete: bool,

    /// Whether or not the user is allowed to access the facilities
    /// requiring the authentication workflow.
    pub allowed: bool,
}

#[cfg(feature = "cli")]
impl Serialize for UserAuthentication {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("UserAuthentication", 3)?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("complete", &self.complete)?;
        s.serialize_field("allowed", &self.allowed)?;
        s.end()
    }
}

#[derive(Accounts)]
pub struct CreateUserAuthentication<'info> {
    /// The user address to be authenticated
    user: AccountInfo<'info>,

    /// The address paying any rent costs
    #[account(mut)]
    payer: Signer<'info>,

    /// The authentication account to be created
    #[account(
        init,
        payer = payer,
        seeds = [user.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<UserAuthentication>(),
    )]
    auth: Account<'info, UserAuthentication>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Authenticate<'info> {
    /// The authentication account for the relevant user to be authenticated
    #[account(mut)]
    auth: Account<'info, UserAuthentication>,

    /// The authority that can authenticate users
    #[cfg_attr(not(feature = "testing"), account(address = authority::ID))]
    authority: Signer<'info>,
}

#[program]
pub mod jet_auth {
    use super::*;

    /// Create a new account that can be used to identify that a
    /// wallet/address is properly authenticated to perform protected actions.
    pub fn create_user_auth(ctx: Context<CreateUserAuthentication>) -> Result<()> {
        let auth = &mut ctx.accounts.auth;

        auth.owner = ctx.accounts.user.key();
        auth.complete = false;
        auth.allowed = false;

        emit!(AuthAccountCreated { user: auth.owner });

        Ok(())
    }

    /// Authenticate a user address
    pub fn authenticate(ctx: Context<Authenticate>) -> Result<()> {
        let auth = &mut ctx.accounts.auth;

        auth.complete = true;
        auth.allowed = true;

        emit!(Authenticated { user: auth.owner });

        Ok(())
    }
}

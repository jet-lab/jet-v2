use crate::control::state::BondManager;
use anchor_lang::{
    prelude::{AccountLoader, Context, CpiContext, Program, Result},
    ToAccountInfo,
};
use anchor_spl::token::{burn, mint_to, transfer, Burn, MintTo, Token, Transfer};

pub trait BondManagerProvider<'info> {
    fn bond_manager(&self) -> AccountLoader<'info, BondManager>;
}

pub trait TokenProgramProvider<'info> {
    fn token_program(&self) -> Program<'info, Token>;
}

/// Deal with tokens owned by the bond manager
pub trait BondTokenManager<'info>:
    BondManagerProvider<'info> + TokenProgramProvider<'info>
{
    /// Mints tokens from a mint owned by the bond manager
    fn mint(
        &self,
        mint: impl ToAccountInfo<'info>,
        to: impl ToAccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        mint_to(
            CpiContext::new(
                self.token_program().to_account_info(),
                MintTo {
                    mint: mint.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&self.bond_manager().load()?.authority_seeds()]),
            amount,
        )
    }

    /// Transfers tokens out of a vault owned by the bond manager
    fn withdraw(
        &self,
        from: impl ToAccountInfo<'info>,
        to: impl ToAccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        transfer(
            CpiContext::new(
                self.token_program().to_account_info(),
                Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&self.bond_manager().load()?.authority_seeds()]),
            amount,
        )
    }

    /// Burns tokens from a token account owned by the bond manager
    fn burn_notes(
        &self,
        mint: impl ToAccountInfo<'info>,
        from: impl ToAccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        burn(
            CpiContext::new(
                self.token_program().to_account_info(),
                Burn {
                    mint: mint.to_account_info(),
                    from: from.to_account_info(),
                    authority: self.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&self.bond_manager().load()?.authority_seeds()]),
            amount,
        )
    }
}

impl<'info, T> BondTokenManager<'info> for T where
    T: BondManagerProvider<'info> + TokenProgramProvider<'info>
{
}

impl<'info, T> BondManagerProvider<'info> for Context<'_, '_, '_, '_, T>
where
    T: BondManagerProvider<'info>,
{
    fn bond_manager(&self) -> AccountLoader<'info, BondManager> {
        self.accounts.bond_manager()
    }
}

impl<'info, T> TokenProgramProvider<'info> for Context<'_, '_, '_, '_, T>
where
    T: TokenProgramProvider<'info>,
{
    fn token_program(&self) -> Program<'info, Token> {
        self.accounts.token_program()
    }
}

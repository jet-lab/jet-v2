use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Result;
use futures::Future;
use jet_margin::MarginAccount;
use jet_margin_pool::TokenChange;
use jet_solana_client::util::Key;
use solana_sdk::pubkey::Pubkey;

use crate::ix_builder::*;
use crate::solana::pubkey::OrAta;

use super::{CanInvokeTo, Invoke, MarginInvokeContext};

/// Use MarginInvokeContext to invoke instructions to the margin-pool program
impl<K: Key> MarginInvokeContext<K> {
    /// Deposit into a margin pool from the specified source account, creating
    /// the target position in the margin account if necessary.
    pub fn pool_deposit<IxTx>(
        &self,
        underlying_mint: Pubkey,
        source: Option<Pubkey>,
        target: PoolTargetPosition,
        change: TokenChange,
    ) -> Vec<IxTx>
    where
        Self: CanInvokeTo<IxTx>,
    {
        let pool = MarginPoolIxBuilder::new(underlying_mint);
        let auth = self.authority.address();
        let (target, mut instructions) = self.get_or_create_pool_deposit(underlying_mint, target);
        instructions.push(self.invoke(pool.deposit(
            self.margin_account,
            source.or_ata(&auth, &underlying_mint),
            target,
            change,
        )));
        instructions
    }

    /// Return the address to a pool deposit for this user, including
    /// instructions to create and refresh the position if necessary.
    pub fn get_or_create_pool_deposit<IxTx>(
        &self,
        underlying_mint: Pubkey,
        position: PoolTargetPosition,
    ) -> (Pubkey, Vec<IxTx>)
    where
        Self: CanInvokeTo<IxTx>,
    {
        let pool = MarginPoolIxBuilder::new(underlying_mint);
        let mut instructions = vec![];
        let target = match position {
            PoolTargetPosition::Existing(pos) => pos,
            PoolTargetPosition::NeedNew { pool_oracle, payer } => {
                instructions.extend(self.create_pool_deposit(payer, underlying_mint, pool_oracle));
                get_associated_token_address(&self.margin_account, &pool.deposit_note_mint)
            }
        };
        (target, instructions)
    }

    /// Create and refresh a pool deposit position
    pub fn create_pool_deposit<IxTx>(
        &self,
        payer: Pubkey,
        underlying_mint: Pubkey,
        pool_oracle: Pubkey,
    ) -> Vec<IxTx>
    where
        Self: CanInvokeTo<IxTx>,
    {
        let pool = MarginPoolIxBuilder::new(underlying_mint);
        let auth = self.authority.address();
        let mut instructions = vec![];
        instructions.extend(self.dont_wrap_any(create_deposit_account_and_position(
            self.margin_account,
            self.airspace,
            auth,
            payer,
            pool.deposit_note_mint,
        )));
        instructions
            .push(self.invoke(pool.margin_refresh_position(self.margin_account, pool_oracle)));
        instructions
    }
}

/// An instruction needs to allocate a non-zero balance into a pool position.
/// This type represents whether or not the position exists:
/// - if so, it provides the address of the token account for that position.
/// - if not, it provides the data that will be necessary to create and refresh
///   the position, so it can successfully acquire a balance.
pub enum PoolTargetPosition {
    /// The position already exists at the provided token account
    Existing(Pubkey),
    /// The position does not exist. Use this data to create and refresh it.
    NeedNew {
        /// needed to refresh the position
        pool_oracle: Pubkey,
        /// funds the creation of the token account
        payer: Pubkey,
    },
}

impl PoolTargetPosition {
    /// common pattern to figure out what information is needed to target a pool
    /// position.
    pub async fn new<Fut: Future<Output = Result<Pubkey, E>>, E>(
        margin_account: &MarginAccount,
        position_token_mint: &Pubkey,
        payer: &Pubkey,
        pool_oracle: Fut,
    ) -> Result<PoolTargetPosition, E> {
        Ok(match margin_account.get_position(position_token_mint) {
            Some(pos) => PoolTargetPosition::Existing(pos.address),
            None => PoolTargetPosition::NeedNew {
                pool_oracle: pool_oracle.await?,
                payer: *payer,
            },
        })
    }
}

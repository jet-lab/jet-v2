use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use futures::future::try_join_all;
use jet_fixed_term::{
    margin::state::{MarginUser, TermLoan},
    tickets::state::TermDeposit,
};
use jet_instructions::fixed_term::{derive, FixedTermIxBuilder};
use jet_simulation::SolanaRpcClient;
use jet_solana_client::{
    rpc::AccountFilter,
    transaction::{TransactionBuilder, WithSigner},
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair};
use thiserror::Error;

use crate::solana::transaction::SendTransactionBuilder;

type KeyAccount<T> = (Pubkey, T);

pub struct AutoRollServicer {
    ix: FixedTermIxBuilder,
    rpc: Arc<dyn SolanaRpcClient>,
}

impl AutoRollServicer {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>, ix: FixedTermIxBuilder) -> Self {
        Self { ix, rpc }
    }

    pub async fn service_all(&self) -> Result<()> {
        let mut jobs = vec![];
        for user in self.fetch_users().await? {
            jobs.push(self.service_user(user))
        }
        try_join_all(jobs).await?;
        Ok(())
    }

    pub async fn service_forever(&self) {
        loop {
            let res = self.service_all().await;
            match res {
                Ok(_) => continue,
                Err(e) => {
                    tracing::error!(
                        "encountered error while servicing market [{}], error: {}",
                        self.ix.market(),
                        e
                    );
                    continue;
                }
            }
        }
    }

    async fn service_user(&self, user: KeyAccount<MarginUser>) -> Result<()> {
        let mut ixns = vec![];
        self.with_service_loans(&user, &mut ixns).await?;
        self.with_service_deposits(&user, &mut ixns).await?;

        self.bundle_and_send(ixns).await
    }

    async fn with_service_loans(
        &self,
        user: &KeyAccount<MarginUser>,
        ixns: &mut Vec<Instruction>,
    ) -> Result<()> {
        if user.1.lend_roll_config.is_none() {
            return Ok(());
        }
        let loans = self.get_active_loans(user).await?;

        let current_time = self.rpc.get_clock().await?.unix_timestamp;
        let mut next_debt_seqno = user.1.debt().next_new_loan_seqno();
        for (loan_key, loan) in loans {
            if loan.strike_timestamp + user.1.borrow_roll_config.as_ref().unwrap().roll_tenor as i64
                >= current_time
            {
                ixns.push(self.ix.auto_roll_borrow_order(
                    user.1.margin_account,
                    loan_key,
                    loan.payer,
                    next_debt_seqno,
                ));
                next_debt_seqno += 1;
            }
        }
        Ok(())
    }

    async fn with_service_deposits(
        &self,
        user: &KeyAccount<MarginUser>,
        ixns: &mut Vec<Instruction>,
    ) -> Result<()> {
        if user.1.borrow_roll_config.is_none() {
            return Ok(());
        }
        let deposits = self.get_active_deposits(user).await?;
        let current_time = self.rpc.get_clock().await?.unix_timestamp;
        let mut next_deposit_seqno = user.1.assets().next_new_deposit_seqno();
        for (deposit_key, deposit) in deposits {
            if deposit.matures_at >= current_time {
                ixns.push(self.ix.auto_roll_lend_order(
                    user.1.margin_account,
                    deposit_key,
                    deposit.payer,
                    next_deposit_seqno,
                ));
                next_deposit_seqno += 1;
            }
        }
        Ok(())
    }

    async fn fetch_users(&self) -> Result<Vec<KeyAccount<MarginUser>>> {
        let users = self
            .rpc
            .get_program_accounts(
                &jet_fixed_term::ID,
                vec![AccountFilter::DataSize(
                    8 + std::mem::size_of::<MarginUser>(),
                )],
            )
            .await?
            .into_iter()
            .filter_map(
                |(k, a)| match MarginUser::try_deserialize(&mut a.data.as_ref()) {
                    Ok(u) => {
                        if u.borrow_roll_config.is_none() && u.lend_roll_config.is_none() {
                            None
                        } else {
                            Some((k, u))
                        }
                    }
                    Err(_) => None,
                },
            )
            .collect();

        Ok(users)
    }

    async fn get_active_loans(
        &self,
        user: &KeyAccount<MarginUser>,
    ) -> Result<Vec<KeyAccount<TermLoan>>> {
        let loan_keys = user
            .1
            .debt()
            .active_loans()
            .map(|n| derive::term_loan(&user.1.market, &user.0, n))
            .collect::<Vec<_>>();
        self.load_accounts::<TermLoan>(&loan_keys).await
    }

    async fn get_active_deposits(
        &self,
        user: &KeyAccount<MarginUser>,
    ) -> Result<Vec<KeyAccount<TermDeposit>>> {
        let deposit_keys = user
            .1
            .assets()
            .active_deposits()
            .map(|n| derive::term_deposit(&user.1.market, &user.0, n))
            .collect::<Vec<_>>();
        self.load_accounts::<TermDeposit>(&deposit_keys).await
    }

    async fn load_accounts<T: AccountDeserialize>(
        &self,
        keys: &[Pubkey],
    ) -> Result<Vec<KeyAccount<T>>> {
        let accs = self
            .rpc
            .get_multiple_accounts(keys)
            .await?
            .into_iter()
            .zip(keys.iter())
            .filter_map(|(a, k)| match T::try_deserialize(&mut a?.data.as_ref()) {
                Ok(a) => Some((*k, a)),
                Err(_) => None,
            })
            .collect();
        Ok(accs)
    }

    async fn bundle_and_send(&self, ix: Vec<Instruction>) -> Result<()> {
        self.rpc
            .send_and_confirm_condensed_in_order(vec![
                TransactionBuilder::from(ix).with_signers(&[self.clone_payer()])
            ])
            .await?;
        Ok(())
    }

    fn clone_payer(&self) -> Keypair {
        Keypair::from_base58_string(&self.rpc.payer().to_base58_string())
    }
}

#[derive(Error, Debug)]
pub enum ServicerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] anyhow::Error),

    #[error("anchor error: {0}")]
    Anchor(#[from] anchor_lang::error::Error),

    #[error("failed to fetch the instruction builder for market: {0}")]
    MissingIxBuilder(Pubkey),
}
type Result<T> = std::result::Result<T, ServicerError>;

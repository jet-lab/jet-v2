use std::{sync::Arc, time::Duration};

use anchor_lang::AccountDeserialize;
use futures::future::join_all;
use jet_fixed_term::{
    margin::state::{MarginUser, TermLoan, TermLoanFlags},
    tickets::state::{TermDeposit, TermDepositFlags},
};
use jet_instructions::{
    fixed_term::{derive, FixedTermIxBuilder},
    margin::accounting_invoke,
};
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
    min_order_size: u64,
}

impl AutoRollServicer {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>, ix: FixedTermIxBuilder, min_order_size: u64) -> Self {
        Self {
            ix,
            rpc,
            min_order_size,
        }
    }

    pub async fn service_all(&self) {
        let users = match self.fetch_users().await {
            Ok(u) => u.into_iter().map(|u| self.service_user(u)),
            Err(e) => {
                tracing::warn!("encountered error fetching users: [{e}]");
                return;
            }
        };
        let num_users = users.len();
        if num_users > 0 {
            tracing::trace!(
                "attemtping to service [{}] users for market [{}]",
                num_users,
                self.ix.market()
            );
        }

        for res in join_all(users).await {
            match res {
                Err(e) => tracing::warn!("encountered error while servicing users: [{e}]"),
                Ok(_) => {
                    tracing::trace!("successfully serviced [{num_users}] users");
                    continue;
                }
            }
        }
    }

    pub async fn service_forever(&self, delay: Duration) {
        tracing::trace!("starting servicer loop");
        loop {
            self.service_all().await;
            tokio::time::sleep(delay).await;
        }
    }

    async fn service_user(&self, user: KeyAccount<MarginUser>) -> Result<()> {
        tracing::trace!("servicing user [{}]", user.0);

        let mut ixns = vec![];
        self.with_service_loans(&user, &mut ixns).await?;
        self.with_service_deposits(&user, &mut ixns).await?;
        if !ixns.is_empty() {
            tracing::debug!(
                "sending [{}] instructions to service user [{}]",
                ixns.len(),
                user.0
            );
        }
        self.bundle_and_send(ixns).await
    }

    async fn with_service_loans(
        &self,
        user: &KeyAccount<MarginUser>,
        ixns: &mut Vec<Instruction>,
    ) -> Result<()> {
        if user.1.borrow_roll_config.is_none() {
            return Ok(());
        }
        tracing::trace!(
            "fetching active loans for user [{}] in market [{}]",
            user.0,
            self.ix.market()
        );
        let loans = self.get_active_loans(user).await?;
        let current_time = self.rpc.get_clock().await?.unix_timestamp;

        if !loans.is_empty() {
            tracing::trace!(
                "found [{}] active loans for user [{}] at timestamp [{}]",
                loans.len(),
                user.0,
                current_time
            );
        }

        let mut next_debt_seqno = user.1.debt().next_new_loan_seqno();
        let mut next_unpaid_loan_seqno =
            user.1.debt().next_term_loan_to_repay().unwrap_or_default() + 1;
        for (loan_key, loan) in loans {
            if !loan.flags.contains(TermLoanFlags::AUTO_ROLL) {
                continue;
            }
            if loan.balance < self.min_order_size {
                continue;
            }
            if loan.strike_timestamp + user.1.borrow_roll_config.as_ref().unwrap().roll_tenor as i64
                >= current_time
            {
                tracing::debug!("attempting to auto-borrow for loan [{}]", loan_key,);
                let auto_borrow = self.ix.auto_roll_borrow_order(
                    user.1.margin_account,
                    loan_key,
                    loan.payer,
                    next_debt_seqno,
                    next_unpaid_loan_seqno,
                );
                ixns.push(accounting_invoke(
                    self.ix.airspace(),
                    user.1.margin_account,
                    auto_borrow,
                ));
                next_debt_seqno += 1;
                next_unpaid_loan_seqno += 1;
            }
        }
        Ok(())
    }

    async fn with_service_deposits(
        &self,
        user: &KeyAccount<MarginUser>,
        ixns: &mut Vec<Instruction>,
    ) -> Result<()> {
        if user.1.lend_roll_config.is_none() {
            return Ok(());
        }
        tracing::trace!(
            "fetching active deposits for user [{}] in market [{}]",
            user.0,
            self.ix.market()
        );
        let deposits = self.get_active_deposits(user).await?;
        let current_time = self.rpc.get_clock().await?.unix_timestamp;
        if !deposits.is_empty() {
            tracing::trace!(
                "found [{}] active deposits for user [{}] at timestamp [{}]",
                deposits.len(),
                user.0,
                current_time
            );
        }
        let mut next_deposit_seqno = user.1.assets().next_new_deposit_seqno();
        for (deposit_key, deposit) in deposits {
            if !deposit.flags.contains(TermDepositFlags::AUTO_ROLL) {
                continue;
            }
            if deposit.amount < self.min_order_size {
                continue;
            }
            if deposit.matures_at <= current_time {
                tracing::debug!("attempting to auto-lend for deposit [{}]", deposit_key);
                let auto_lend = self.ix.auto_roll_lend_order(
                    user.1.margin_account,
                    deposit_key,
                    deposit.payer,
                    next_deposit_seqno,
                );
                ixns.push(accounting_invoke(
                    self.ix.airspace(),
                    user.1.margin_account,
                    auto_lend,
                ));
                next_deposit_seqno += 1;
            }
        }
        Ok(())
    }

    async fn fetch_users(&self) -> Result<Vec<KeyAccount<MarginUser>>> {
        tracing::trace!("fetching users from market [{}]", self.ix.market());
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
                        if u.market != self.ix.market()
                            || (u.borrow_roll_config.is_none() && u.lend_roll_config.is_none())
                        {
                            None
                        } else {
                            Some((k, u))
                        }
                    }
                    Err(_) => {
                        tracing::warn!("failed to deserialize margin user [{k}]");
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        if !users.is_empty() {
            tracing::debug!(
                "found [{}] margin users in market [{}]",
                users.len(),
                self.ix.market()
            );
        }
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
        let mut loans = self.load_accounts::<TermLoan>(&loan_keys).await?;
        loans.sort_by(|a, b| {
            a.1.sequence_number
                .partial_cmp(&b.1.sequence_number)
                .unwrap()
        });
        Ok(loans)
    }

    async fn get_active_deposits(
        &self,
        user: &KeyAccount<MarginUser>,
    ) -> Result<Vec<KeyAccount<TermDeposit>>> {
        let deposit_keys = user
            .1
            .assets()
            .active_deposits()
            .map(|n| derive::term_deposit(&user.1.market, &user.1.margin_account, n))
            .collect::<Vec<_>>();
        let mut deposits = self.load_accounts::<TermDeposit>(&deposit_keys).await?;
        deposits.sort_by(|a, b| {
            a.1.sequence_number
                .partial_cmp(&b.1.sequence_number)
                .unwrap()
        });
        Ok(deposits)
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
                Err(_) => {
                    tracing::warn!("failed to deserialize account [{k}]");
                    None
                }
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

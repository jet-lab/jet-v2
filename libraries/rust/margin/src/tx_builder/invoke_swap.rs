use jet_margin_pool::TokenChange;
use jet_solana_client::{
    cat,
    transaction::InstructionBundle,
    util::{pubkey::OrAta, Key},
};
use solana_sdk::{compute_budget::ComputeBudgetInstruction, pubkey::Pubkey};

use crate::ix_builder::*;

use super::{MarginInvokeContext, PoolTargetPosition};

/// Use MarginInvokeContext to invoke instructions to the margin-pool program
impl<K: Key + Clone> MarginInvokeContext<K> {
    /// Transaction to swap one token for another.
    pub fn swap(
        &self,
        addr: SplSwap,
        source: Option<Pubkey>,
        target: PoolTargetPosition,
        change: TokenChange,
        minimum_amount_out: u64,
    ) -> Vec<InstructionBundle<K>> {
        let source_pool = MarginPoolIxBuilder::new(addr.token_a);
        let source_position = source.or_ata(&self.margin_account, &source_pool.deposit_note_mint);
        let (target_position, mut transactions) =
            self.get_or_create_pool_deposit(addr.token_b, target);

        transactions.push(cat![
            self.direct(ComputeBudgetInstruction::set_compute_unit_limit(400_000)),
            self.invoke(pool_spl_swap(
                &addr,
                &self.airspace,
                &self.margin_account,
                &addr.token_a,
                &addr.token_b,
                Some(source_position),
                Some(target_position),
                change.kind,
                change.tokens,
                minimum_amount_out,
            )),
        ]);

        transactions
    }
}

use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;
use spl_token_swap::state::SwapV1;

use jet_instructions::margin_swap::{pool_spl_swap, SplSwap};
use jet_margin_pool::ChangeKind;
use jet_program_common::programs::ORCA_V2;

use crate::{
    bail,
    client::{ClientResult, ClientState},
    margin::MarginAccountClient,
};

/// Client for interacting with swap protocols, from the perspective of a margin account
pub struct MarginAccountSwapsClient {
    client: Arc<ClientState>,
    account: MarginAccountClient,
}

impl MarginAccountSwapsClient {
    pub fn new(account: MarginAccountClient) -> Self {
        Self {
            client: account.client.clone(),
            account,
        }
    }

    /// Swap tokens in a margin pool, using Orca V2
    ///
    /// # Parameters
    ///
    /// * `swap_pool` - The swap pool to use
    /// * `source_token` - The source token to be exchanged
    /// * `target_token` - The desired token
    /// * `amount` - The amount of the position to exchange, if `None` then swaps entire position.
    /// * `minimum_amount_out` - Limit the possible fill price
    pub async fn orca_v2_swap(
        &self,
        swap_pool: &Pubkey,
        source_token: &Pubkey,
        target_token: &Pubkey,
        amount: Option<u64>,
        minimum_amount_out: u64,
    ) -> ClientResult<()> {
        let swap_info = self.get_orca_v2_swap(swap_pool)?;
        let mut instructions = vec![];

        // Ensure transit accounts exist
        instructions.extend(
            (!self.account.has_position(source_token))
                .then(|| self.account.builder.create_deposit_position(*source_token)),
        );
        instructions.extend(
            (!self.account.has_position(target_token))
                .then(|| self.account.builder.create_deposit_position(*target_token)),
        );

        // Ensure there is a position to deposit the target tokens into
        let target_deposit_note = self.account.pool(target_token).builder.deposit_note_mint;

        instructions.extend(
            (!self.account.has_position(&target_deposit_note))
                .then(|| self.account.builder.register_position(target_deposit_note)),
        );

        // Swap everything if amount not specified
        let (withdrawal_change_kind, withdrawal_amount) = match amount {
            Some(amount) => (ChangeKind::ShiftBy, amount),
            None => (ChangeKind::SetTo, 0),
        };

        instructions.push(pool_spl_swap(
            &swap_info,
            &self.account.airspace(),
            &self.account.address,
            source_token,
            target_token,
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        ));

        self.client.send(&instructions).await
    }

    fn get_orca_v2_swap(&self, swap_pool: &Pubkey) -> ClientResult<SplSwap> {
        let swap = match self.client.state().get::<SwapV1>(swap_pool) {
            Some(swap) => swap,
            None => bail!("no swap pool found in cache with address {swap_pool}"),
        };

        Ok(SplSwap {
            program: ORCA_V2,
            address: *swap_pool,
            pool_mint: swap.pool_mint,
            token_a: swap.token_a_mint,
            token_b: swap.token_b_mint,
            token_a_vault: swap.token_a,
            token_b_vault: swap.token_b,
            fee_account: swap.pool_fee_account,
        })
    }
}

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_sdk::pubkey::Pubkey;
use spl_token_swap::state::SwapV1;

use jet_instructions::{
    margin::derive_position_token_account,
    margin_swap::{pool_spl_swap, MarginSwapRouteIxBuilder, SplSwap, SwapContext},
    openbook::create_open_orders,
};
use jet_margin_pool::{ChangeKind, TokenChange};
use jet_program_common::programs::{OPENBOOK, ORCA_V2};

use crate::{
    bail,
    client::{ClientResult, ClientState},
    margin::MarginAccountClient,
    state::dexes::DexState,
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

    /// Swap tokens with a given path
    pub async fn route_swap(
        &self,
        path: &[SwapStep],
        in_amount: u64,
        minimum_amount_out: u64,
    ) -> ClientResult<()> {
        let mut instructions = vec![];

        let source_token = path[0].from_token;
        let dest_token = path[path.len() - 1].to_token;
        let mut builder = MarginSwapRouteIxBuilder::new(
            SwapContext::MarginPositions,
            self.account.address(),
            source_token,
            dest_token,
            TokenChange::shift(in_amount),
            minimum_amount_out,
        );

        for step in path {
            self.account
                .with_deposit_position(&step.to_token, &mut instructions)
                .await?;

            let Some(dex_state) = self.client.state().get::<DexState>(&step.swap_pool) else {
                bail!("unknown swap pool {}", step.swap_pool);
            };

            if step.program == OPENBOOK {
                let (create_open_orders_ix, open_orders_address) = create_open_orders(
                    self.account.address(),
                    step.swap_pool,
                    self.client.signer(),
                    &OPENBOOK,
                );

                if !self.client.account_exists(&open_orders_address).await? {
                    instructions.push(self.account.builder.adapter_invoke(create_open_orders_ix));
                }
            }

            let swap_accounts = if dex_state.token_a == step.from_token {
                &dex_state.swap_a_to_b_accounts
            } else {
                &dex_state.swap_b_to_a_accounts
            };

            builder.add_swap_leg(swap_accounts.as_ref(), 0)?;
        }

        builder.finalize()?;

        instructions.push(
            self.account
                .builder
                .adapter_invoke(builder.get_instruction()?),
        );

        self.account.send_with_refresh(&instructions).await
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
            //todo use ata
            (!self.account.has_position(&target_deposit_note))
                .then(|| self.account.builder.register_position(target_deposit_note)),
        );

        // Swap everything if amount not specified
        let (withdrawal_change_kind, withdrawal_amount) = match amount {
            Some(amount) => (ChangeKind::ShiftBy, amount),
            None => (ChangeKind::SetTo, 0),
        };

        let account = self.account.state();
        instructions.push(pool_spl_swap(
            &swap_info,
            &self.account.airspace(),
            &self.account.address,
            source_token,
            target_token,
            Some(
                account
                    .get_position(source_token)
                    .map(|x| x.address)
                    .unwrap_or(derive_position_token_account(
                        //todo use ata
                        &self.account.address,
                        source_token,
                    )),
            ),
            Some(
                account
                    .get_position(target_token)
                    .map(|x| x.address)
                    .unwrap_or(derive_position_token_account(
                        //todo use ata
                        &self.account.address,
                        target_token,
                    )),
            ),
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStep {
    #[serde_as(as = "DisplayFromStr")]
    pub from_token: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub to_token: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub program: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub swap_pool: Pubkey,
}

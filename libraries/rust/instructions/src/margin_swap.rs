use std::collections::HashSet;

use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;

use jet_margin_pool::{ChangeKind, TokenChange};
use jet_margin_swap::{accounts as ix_accounts, SwapRouteDetail, SwapRouteIdentifier};
use jet_margin_swap::{instruction as ix_data, ROUTE_SWAP_MAX_SPLIT, ROUTE_SWAP_MIN_SPLIT};

use crate::margin_pool::MarginPoolIxBuilder;
use crate::IxResult;
use crate::JetIxError;
use crate::{get_metadata_address, margin::derive_position_token_account};

pub use jet_margin_swap::ID as MARGIN_SWAP_PROGRAM;

/// Instruction for using an SPL swap with tokens in a margin pool
#[allow(clippy::too_many_arguments)]
pub fn pool_spl_swap(
    swap_info: &SplSwap,
    _airspace: &Pubkey,
    margin_account: &Pubkey,
    source_token: &Pubkey,
    target_token: &Pubkey,
    withdrawal_change_kind: ChangeKind,
    withdrawal_amount: u64,
    minimum_amount_out: u64,
) -> Instruction {
    let pool_source = MarginPoolIxBuilder::new(*source_token);
    let pool_target = MarginPoolIxBuilder::new(*target_token);

    let transit_source_account = get_associated_token_address(margin_account, source_token);
    let transit_destination_account = get_associated_token_address(margin_account, target_token);

    let source_account =
        derive_position_token_account(margin_account, &pool_source.deposit_note_mint);
    let destination_account =
        derive_position_token_account(margin_account, &pool_target.deposit_note_mint);

    let (vault_into, vault_from) = if *source_token == swap_info.token_a {
        (swap_info.token_a_vault, swap_info.token_b_vault)
    } else {
        (swap_info.token_b_vault, swap_info.token_a_vault)
    };

    let accounts = jet_margin_swap::accounts::MarginSplSwap {
        margin_account: *margin_account,
        source_account,
        destination_account,
        transit_source_account,
        transit_destination_account,
        swap_info: jet_margin_swap::accounts::SwapInfo {
            swap_pool: swap_info.address,
            authority: derive_spl_swap_authority(&swap_info.program, &swap_info.address),
            token_mint: swap_info.pool_mint,
            fee_account: swap_info.fee_account,
            swap_program: swap_info.program,
            vault_into,
            vault_from,
        },
        source_margin_pool: jet_margin_swap::accounts::MarginPoolInfo {
            margin_pool: pool_source.address,
            vault: pool_source.vault,
            deposit_note_mint: pool_source.deposit_note_mint,
        },
        destination_margin_pool: jet_margin_swap::accounts::MarginPoolInfo {
            margin_pool: pool_target.address,
            vault: pool_target.vault,
            deposit_note_mint: pool_target.deposit_note_mint,
        },
        margin_pool_program: jet_margin_pool::ID,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: jet_margin_swap::ID,
        data: jet_margin_swap::instruction::MarginSwap {
            withdrawal_change_kind,
            withdrawal_amount,
            minimum_amount_out,
        }
        .data(),
        accounts,
    }
}

pub struct SplSwap {
    pub program: Pubkey,
    pub address: Pubkey,
    pub pool_mint: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub fee_account: Pubkey,
}

pub fn derive_spl_swap_authority(program: &Pubkey, pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[pool.as_ref()], program).0
}

/// Trait to get required information from a swap pool for the [MarginSwapRouteIxBuilder]
pub trait SwapAccounts {
    /// Convert the pool to a vec of [AccountMeta]
    fn to_account_meta(&self) -> Vec<AccountMeta>;
    /// Determine the pool source and destination tokens
    fn pool_tokens(&self) -> (Pubkey, Pubkey);
    /// The identifier of the route
    fn route_type(&self) -> SwapRouteIdentifier;
}

/// The context which a user wants to swap with. Indicates whether a user wants
/// to use margin pools or token accounts as the source and destination of the swap.
#[derive(Debug, Clone, Copy)]
pub enum SwapContext {
    /// Borrow inputs and deposit outputs using margin pools to swap.
    MarginPool,
    /// Use own margin positions to swap.
    MarginPositions,
}

/// A margin route instruction builder that adds, validates routes, and builds
/// the swap instruction.
pub struct MarginSwapRouteIxBuilder {
    /// The margin account creating the swap
    margin_account: Pubkey,
    /// SPL mint of the left side of the pool
    #[allow(unused)]
    src_token: Pubkey,
    /// SPL mint of the right side of the pool
    dst_token: Pubkey,
    /// Route details
    route_details: [SwapRouteDetail; 3],
    /// Token amounts to borrow/withdraw
    withdrawal_change: TokenChange,
    /// Minimum tokens to receive
    minimum_amount_out: u64,
    /// The gathered accounts of the instruction
    account_metas: Vec<AccountMeta>,
    /// The current destination token in a multi-route swap.
    /// Used to validate the swap chain
    current_route_tokens: Option<(Pubkey, Pubkey)>,
    next_route_index: usize,
    /// Whether this builder is finalized
    is_finalized: bool,
    /// Whether the next swap should be part of a multi route
    expects_multi_route: bool,
    /// SPL token accounts used, so the caller can create ATAs
    spl_token_accounts: HashSet<Pubkey>,
    /// Pool deposit notes used, so the caller can create their accounts if necessary
    pool_note_mints: HashSet<Pubkey>,
    /// The context used for the swap
    swap_context: SwapContext,
    /// Whether this builder has liquidation accounts
    is_liquidation: bool,
}

impl MarginSwapRouteIxBuilder {
    /// Create a new builder for a margin swap.
    /// The swap can have up to 3 steps, e.g. JET > USDC > SOL > mSOL, where each step is a leg.
    ///
    /// To get a transaction, call `finalize()`, then get the instruction via `get_instruction()`.
    pub fn try_new(
        swap_context: SwapContext,
        margin_account: Pubkey,
        src_token: Pubkey,
        dst_token: Pubkey,
        withdrawal_change: TokenChange,
        minimum_amount_out: u64,
    ) -> IxResult<Self> {
        // Withdrawal_change can only be shift_by if not using margin
        if matches!(
            (swap_context, withdrawal_change.kind),
            (SwapContext::MarginPositions, ChangeKind::SetTo)
        ) {
            return Err(JetIxError::SwapIxError(
                "Change can only be ShiftBy when not swapping on margin".to_string(),
            ));
        }

        let mut spl_token_accounts = HashSet::with_capacity(4);
        spl_token_accounts.insert(src_token);
        spl_token_accounts.insert(dst_token);

        let mut pool_note_mints = HashSet::with_capacity(4);

        let account_metas = match swap_context {
            SwapContext::MarginPool => {
                let src_pool = MarginPoolIxBuilder::new(src_token);
                let dst_pool = MarginPoolIxBuilder::new(dst_token);

                pool_note_mints.insert(src_pool.deposit_note_mint);
                pool_note_mints.insert(dst_pool.deposit_note_mint);

                let source_account =
                    derive_position_token_account(&margin_account, &src_pool.deposit_note_mint);
                let destination_account =
                    derive_position_token_account(&margin_account, &dst_pool.deposit_note_mint);

                ix_accounts::RouteSwapPool {
                    margin_account,
                    source_account,
                    destination_account,
                    source_margin_pool: ix_accounts::MarginPoolInfo {
                        margin_pool: src_pool.address,
                        vault: src_pool.vault,
                        deposit_note_mint: src_pool.deposit_note_mint,
                    },
                    destination_margin_pool: ix_accounts::MarginPoolInfo {
                        margin_pool: dst_pool.address,
                        vault: dst_pool.vault,
                        deposit_note_mint: dst_pool.deposit_note_mint,
                    },
                    margin_pool_program: jet_margin_pool::id(),
                    token_program: spl_token::id(),
                }
                .to_account_metas(None)
            }
            SwapContext::MarginPositions => ix_accounts::RouteSwap {
                margin_account,
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
        };
        Ok(Self {
            margin_account,
            src_token,
            dst_token,
            route_details: [Default::default(); 3],
            withdrawal_change,
            minimum_amount_out,
            account_metas,
            current_route_tokens: None,
            next_route_index: 0,
            is_finalized: false,
            expects_multi_route: false,
            spl_token_accounts,
            pool_note_mints,
            swap_context,
            is_liquidation: false,
        })
    }

    /// Whether this builder is for a liquidation
    pub fn is_liquidation(&self) -> bool {
        self.is_liquidation
    }

    /// Set a liquidator
    pub fn set_liquidation(
        &mut self,
        liquidator: Pubkey,
        fee_destination: Option<Pubkey>,
    ) -> IxResult<()> {
        if self.is_liquidation {
            return Err(JetIxError::SwapIxError(
                "A liquidator is already set".to_string(),
            ));
        }
        if self.is_finalized {
            return Err(JetIxError::SwapIxError(
                "Swap route is already finalized".to_string(),
            ));
        }
        if self.current_route_tokens.is_some() {
            return Err(JetIxError::SwapIxError(
                "Swap route not empty, can only add liquidator to empty route".to_string(),
            ));
        }
        let liquidation = get_metadata_address(&liquidator);
        self.account_metas.extend_from_slice(&[
            AccountMeta {
                pubkey: liquidation,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: fee_destination.unwrap(), // TODO: derive when not in tests
                is_signer: false,
                is_writable: true,
            },
        ]);

        self.is_liquidation = true;

        Ok(())
    }

    /// Add a swap leg to the route
    pub fn add_swap_leg<T: SwapAccounts>(&mut self, pool: &T, swap_split: u8) -> IxResult<()> {
        // Check the swap split early
        if swap_split > ROUTE_SWAP_MAX_SPLIT
            || (swap_split > 0 && swap_split < ROUTE_SWAP_MIN_SPLIT)
        {
            return Err(JetIxError::SwapIxError(format!("Invalid swap split, must be >= {ROUTE_SWAP_MIN_SPLIT} and <= {ROUTE_SWAP_MAX_SPLIT}")));
        }
        let (src_token, dst_token) = self.src_dst_tokens(pool)?;
        // Run common checks
        self.verify_addition(&src_token, &dst_token, swap_split)?;

        if !self.expects_multi_route {
            // Add source ATA and pool accounts. Add destination only if this is
            // the first part of a split leg.
            let src_ata = get_associated_token_address(&self.margin_account, &src_token);
            self.account_metas.push(AccountMeta::new(src_ata, false));
        }

        // Add swap pool accounts
        let mut accounts = pool.to_account_meta();
        self.account_metas.append(&mut accounts);

        if !self.expects_multi_route && swap_split > 0 {
            // Add destination ATA
            let dst_ata = get_associated_token_address(&self.margin_account, &dst_token);
            self.account_metas.push(AccountMeta::new(dst_ata, false));
        }

        // Update the route information and persist builder state
        let Some(mut route) = self
            .route_details
            .get_mut(self.next_route_index) else {
                return Err(JetIxError::SwapIxError("Unable to get route detail".to_string()));
            };
        if self.expects_multi_route {
            // This is the second leg of the multi-route
            route.route_b = pool.route_type();
            self.expects_multi_route = false;
            self.next_route_index += 1;
        } else {
            route.route_a = pool.route_type();
            route.split = swap_split;
            if swap_split > 0 {
                self.expects_multi_route = true;
            } else {
                self.next_route_index += 1;
            }
        }
        // Update the current tokens in the swap
        self.current_route_tokens = Some((src_token, dst_token));

        Ok(())
    }

    /// Validate and finalize this swap
    pub fn finalize(&mut self) -> IxResult<()> {
        if self.is_finalized {
            return Err(JetIxError::SwapIxError(
                "Swap route is already finalized".to_string(),
            ));
        }
        // Start with simple condiitions for data that should be present
        if self.next_route_index == 0 {
            return Err(JetIxError::SwapIxError(
                "No swap routes seem to be added".to_string(),
            ));
        }
        if self.expects_multi_route {
            return Err(JetIxError::SwapIxError(
                "Swap incomplete, expected a second part of a swap to be executed as a split"
                    .to_string(),
            ));
        }
        match &self.current_route_tokens {
            None => {
                return Err(JetIxError::SwapIxError(
                    "There should be current route tokens populated in the swap".to_string(),
                ));
            }
            Some((_, b)) => {
                if &self.dst_token != b {
                    return Err(JetIxError::SwapIxError(
                        "Swap does not terminate in the provided destination token".to_string(),
                    ));
                }
            }
        }
        // Add destination ATA
        let dst_ata = get_associated_token_address(&self.margin_account, &self.dst_token);
        self.account_metas.push(AccountMeta::new(dst_ata, false));

        // Safe to finalize
        self.is_finalized = true;
        Ok(())
    }

    /// Get the instruction of the swap, which the caller should wrap with an invoke action
    pub fn get_instruction(&self) -> IxResult<Instruction> {
        // Check if finalized
        if !self.is_finalized {
            return Err(JetIxError::SwapIxError(
                "Can only get instruction when the builder is finalized".to_string(),
            ));
        }
        Ok(Instruction {
            program_id: jet_margin_swap::id(),
            accounts: self.account_metas.clone(),
            data: match self.swap_context {
                SwapContext::MarginPool => ix_data::RouteSwapPool {
                    withdrawal_change_kind: self.withdrawal_change.kind,
                    withdrawal_amount: self.withdrawal_change.tokens,
                    minimum_amount_out: self.minimum_amount_out,
                    swap_routes: self.route_details,
                    is_liquidation: self.is_liquidation,
                }
                .data(),
                SwapContext::MarginPositions => ix_data::RouteSwap {
                    amount_in: self.withdrawal_change.tokens,
                    minimum_amount_out: self.minimum_amount_out,
                    swap_routes: self.route_details,
                    is_liquidation: self.is_liquidation,
                }
                .data(),
            },
        })
    }

    /// Get the pool note mints that are used in the instruction
    pub fn get_pool_note_mints(&self) -> &HashSet<Pubkey> {
        &self.pool_note_mints
    }

    /// Get SPL token accounts used in the transfer
    pub fn get_spl_token_mints(&self) -> &HashSet<Pubkey> {
        &self.spl_token_accounts
    }

    /// Determine the source and destination pool mints
    fn src_dst_tokens<T: SwapAccounts>(&self, pool: &T) -> IxResult<(Pubkey, Pubkey)> {
        let (mint_a, mint_b) = pool.pool_tokens();
        let next_src_token = self
            .current_route_tokens
            .map(|(a, b)| {
                // If splitting a route, the expected source is a, else b
                if self.expects_multi_route {
                    a
                } else {
                    b
                }
            })
            .unwrap_or(self.src_token);
        if mint_a == next_src_token {
            Ok((mint_a, mint_b))
        } else if mint_b == next_src_token {
            Ok((mint_b, mint_a))
        } else {
            Err(JetIxError::SwapIxError(format!(
                "Expected a swap pool that has {next_src_token} as one of its token mints"
            )))
        }
    }

    /// Verify that the swap can be added
    fn verify_addition(
        &self,
        src_token: &Pubkey,
        dst_token: &Pubkey,
        swap_split: u8,
    ) -> IxResult<()> {
        // If we are on the last index, we can only get a split
        if self.is_finalized {
            return Err(JetIxError::SwapIxError(
                "Cannot add route to a finalized swap".to_string(),
            ));
        }
        if self.next_route_index > 2 {
            return Err(JetIxError::SwapIxError(
                "Cannot add more routes".to_string(),
            ));
        }
        if self.expects_multi_route && swap_split > 0 {
            return Err(JetIxError::SwapIxError("The next route is expected to be a second leg of a multi swap, do not specify percentage split".to_string()));
        }
        // Check that the source token agrees with the expected next token
        if let Some((a, b)) = &self.current_route_tokens {
            // If on a multi-hop, the source and desitnation must agree, otherwise source = destination
            if self.expects_multi_route && (a != src_token || b != dst_token) {
                return Err(JetIxError::SwapIxError(
                    "Source and destination tokens must be the same in a split-route swap"
                        .to_string(),
                ));
            }
            if !self.expects_multi_route && b != src_token {
                return Err(JetIxError::SwapIxError(
                    "The source token must be the same as the expected destination".to_string(),
                ));
            }
        }

        Ok(())
    }
}

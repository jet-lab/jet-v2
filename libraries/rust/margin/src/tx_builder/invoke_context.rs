//! This module only defines the generic code for executing margin invocations.
//! Other modules define ways to use this context to invoke specific adapters.

use jet_instructions::margin::{accounting_invoke, adapter_invoke, liquidator_invoke};
use jet_solana_client::{
    signature::NeedsSignature,
    transaction::{InstructionBundle, WithSigner},
    util::{data::With, Key},
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Minimum information necessary to wrap an instruction in a margin invoke and
/// sign the transaction. Simpler alternative to MarginTxBuilder, to minimize
/// dependencies.
///
/// Data that is needed for a MarginTxBuilder, but not this:
/// - an RPC client
/// - signer keypair
/// - margin account seed
/// - margin account owner
/// - payer
pub struct MarginInvokeContext<K> {
    /// The airspace where the margin account is authorized.
    pub airspace: Pubkey,
    /// The margin account that will wrap the instruction.
    pub margin_account: Pubkey,
    /// The signer who may authorize actions for the margin account.
    pub authority: K,
    /// Is the authority a liquidator?
    pub is_liquidator: bool,
}

impl<K: Key> MarginInvokeContext<K> {
    /// Invoke a margin adapter through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// bool indicates if a signature is needed from the account authority.
    fn invoke_ix(&self, inner: Instruction) -> (Instruction, bool) {
        let MarginInvokeContext {
            airspace,
            margin_account,
            authority,
            is_liquidator,
        } = self;
        let ix = if inner.needs_signature(*margin_account) {
            if *is_liquidator {
                liquidator_invoke(*airspace, authority.address(), *margin_account, inner)
            } else {
                adapter_invoke(*airspace, authority.address(), *margin_account, inner)
            }
        } else {
            accounting_invoke(*airspace, *margin_account, inner)
        };
        let wrapped_needs_signature = ix.needs_signature(self.authority.address());

        (ix, wrapped_needs_signature)
    }
}

impl<K: Key + Clone> MarginInvokeContext<K> {
    /// Invoke margin adapters through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer if any instructions need it
    pub fn invoke(&self, inner: Instruction) -> InstructionBundle<K> {
        let (wrapped, needs_signature) = self.invoke_ix(inner);
        if needs_signature {
            wrapped.with_signer(self.authority.clone())
        } else {
            wrapped.into()
        }
    }

    /// Applies `invoke` individually to each instruction and returns a vec of results
    pub fn invoke_each(&self, inners: Vec<Instruction>) -> Vec<InstructionBundle<K>> {
        inners.into_iter().map(|ix| self.invoke(ix)).collect()
    }

    /// Individually invokes each instruction and combines it into a single
    /// bundle.
    ///
    /// This is a more efficient alterative to joining the result from
    /// invoke_each:
    /// ```rust ignore
    /// self.invoke_joined(ixs) == self.invoke_each(ixs).ijoin()
    /// ```
    pub fn invoke_joined(&self, inners: Vec<Instruction>) -> InstructionBundle<K> {
        let mut needs_signature = false;
        let mut all_wrapped = vec![];
        for inner in inners {
            let wrapped = self.invoke_ix(inner);
            all_wrapped.push(wrapped.0);
            needs_signature |= wrapped.1;
        }
        if needs_signature {
            all_wrapped.with_signer(self.authority.clone())
        } else {
            all_wrapped.into()
        }
    }

    /// executes the same conversion into an InstructionBundle<K> as invoke but
    /// without actually wrapping it in a margin invocation.
    pub fn direct(&self, inner: Instruction) -> InstructionBundle<K> {
        inner.into()
    }

    /// executes the same conversion into an InstructionBundle<K> as invoke but
    /// without actually wrapping it in a margin invocation.
    pub fn direct_each(&self, ixs: Vec<Instruction>) -> Vec<InstructionBundle<K>> {
        ixs.into_iter().map(|ix| ix.into()).collect()
    }
}

/// Extension methods for Instruction and TransactionBuilder.
///
/// These traits are for improving the readability of operations with
/// collections of instructions and chained method calls.
///
/// Inverts the receiver for methods of MarginTestContext, so Instruction or
/// Vec<Instruction> can be the receiver. This means you can chain method calls
/// in a builder pattern as a clear sequence of steps:
/// ```ignore
/// ix_builder
///     .borrow(100)
///     .invoke(ctx)
///     .send_and_confirm(rpc)
/// ```
pub mod invoke_into {
    use super::*;

    /// Defines the way to unpack some type, invoke any containing instructions
    /// through margin, and pack it back into the same type.
    pub trait InvokeEachInto<K>: Sized {
        /// Only wraps the included instruction(s), without signing anything.  
        /// bool indicates if a signature is needed
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool);

        /// Separately invokes each instruction into a desired type.
        fn invoke_each_into(self, ctx: &MarginInvokeContext<K>) -> Vec<InstructionBundle<K>>;
    }

    impl<K: Key + Clone> InvokeEachInto<K> for Instruction {
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
            ctx.invoke_ix(self)
        }

        fn invoke_each_into(self, ctx: &MarginInvokeContext<K>) -> Vec<InstructionBundle<K>> {
            vec![ctx.invoke(self)]
        }
    }

    impl<K: Key + Clone> InvokeEachInto<K> for Vec<Instruction> {
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
            self.into_iter()
                .map(|ix| ctx.invoke_ix(ix))
                .fold((vec![], false), |(ixs, any_need), (ix, this_needs)| {
                    (ixs.with(ix), any_need | this_needs)
                })
        }

        fn invoke_each_into(self, ctx: &MarginInvokeContext<K>) -> Vec<InstructionBundle<K>> {
            ctx.invoke_each(self)
        }
    }

    /// Defines the way to unpack some type, invoke any containing instructions
    /// through margin, and pack it all into a single TransactionBuilder.
    pub trait InvokeInto: Sized {
        /// Invoke a margin adapter through a margin account using whichever wrapper
        /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke. If
        /// there are multiple instructions, they are combined into a single
        /// TransactionBuilder
        fn invoke_into<K: Key + Clone>(self, ctx: &MarginInvokeContext<K>) -> InstructionBundle<K>;
    }

    impl InvokeInto for Instruction {
        fn invoke_into<K: Key + Clone>(self, ctx: &MarginInvokeContext<K>) -> InstructionBundle<K> {
            ctx.invoke(self)
        }
    }

    impl InvokeInto for Vec<Instruction> {
        fn invoke_into<K: Key + Clone>(self, ctx: &MarginInvokeContext<K>) -> InstructionBundle<K> {
            ctx.invoke_joined(self)
        }
    }
}

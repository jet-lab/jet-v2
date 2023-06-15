//! This module only defines the generic code for executing margin invocations.
//! Other modules define ways to use this context to invoke specific adapters.

use jet_instructions::margin::{accounting_invoke, adapter_invoke, liquidator_invoke};
use jet_solana_client::{
    signature::NeedsSignature,
    transaction::{TransactionBuilder, WithSigner},
    util::{data::With, keypair::KeypairExt, Key},
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair};

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
///
/// If K cannot sign, then it won't actually sign transactions, but it can still
/// wrap instructions.
pub struct MarginInvokeContext<K: Key> {
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
    /// bool indicates if a signature is needed.
    fn invoke_ix(&self, inner: Instruction) -> (Instruction, bool) {
        let MarginInvokeContext {
            airspace,
            margin_account,
            authority,
            is_liquidator,
        } = self;
        let needs_signature = inner.needs_signature(*margin_account);
        (
            if inner.needs_signature(*margin_account) {
                if *is_liquidator {
                    liquidator_invoke(*airspace, authority.address(), *margin_account, inner)
                } else {
                    adapter_invoke(*airspace, authority.address(), *margin_account, inner)
                }
            } else {
                accounting_invoke(*airspace, *margin_account, inner)
            },
            needs_signature,
        )
    }
}

impl MarginInvokeContext<Keypair> {
    /// Invoke margin adapters through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer if any instructions need it
    pub fn invoke_many(&self, inners: Vec<Instruction>) -> TransactionBuilder {
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
}

/// Methods for MarginInvokeContext to invoke instructions through a margin
/// account.
///
/// These methods exist in a separate trait from CanInvokeTo to make it cleaner
/// to call them. Without this trait, there would be some situations where you
/// would need to cast the context to CanInvokeTo which is pointlessly verbose
/// and hard to read.
pub trait Invoke {
    /// Invoke a margin adapter through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer if needed and if the return type supports it.
    fn invoke<T>(&self, inner: Instruction) -> T
    where
        Self: CanInvokeTo<T>,
    {
        self.__private_invoke(inner)
    }

    /// Invoke margin adapters through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer for any transactions that need it and if the return
    /// type supports it.
    fn invoke_each<T>(&self, ixs: Vec<Instruction>) -> Vec<T>
    where
        Self: CanInvokeTo<T>,
    {
        ixs.into_iter().map(|ix| self.invoke(ix)).collect()
    }

    /// executes the same type conversion as invoke but without actually
    /// wrapping it in a margin invocation.
    fn dont_wrap<T>(&self, inner: Instruction) -> T
    where
        Self: CanInvokeTo<T>,
    {
        self.__private_dont_wrap(inner)
    }

    /// executes the same type conversion as invoke_each but without actually
    /// wrapping it in a margin invocation.
    fn dont_wrap_any<T>(&self, ixs: Vec<Instruction>) -> Vec<T>
    where
        Self: CanInvokeTo<T>,
    {
        ixs.into_iter().map(|ix| self.dont_wrap(ix)).collect()
    }
}
impl<T> Invoke for T {}

/// This trait represents the ability to convert an instruction into one that is
/// invoked through a margin account, and to output the result as an arbitrary
/// type.
///
/// This is a behavior that we want in MarginInvokeContext. Instead of being a
/// direct method, we implement it through a trait, to enable calling functions
/// to be generic.
///
/// Some function might accept a MarginInvokeContext as a parameter, and return
/// a transaction or instruction as a return type. To maximize the flexibility
/// of that function, you may want both of these to be generic:
/// - the type of authority key (pubkey or keypair) in the MarginInvokeContext
/// - the return type of the function (either Instruction or TransactionBuilder)
///
/// If this trait's method were just an ordinary method of MarginInvokeContext,
/// it would not be possible to make this hypothetical function so generic, for
/// two reasons:
/// - This method would actually need to be multiple methods with different
///   names because you cannot have different implementations that are treated
///   as the same method, unless it is a trait method.
/// - There is no way to express the requirement in a function signature that
///   you need a particular combination of types that have a particular method
///   implemented for them, except with trait bounds.
///
/// The function would be forced to specify a concrete input or output type.
///
/// By dedicating a trait to this method, we allow the function to be generic
/// over both its input and output types. The function only needs to express a
/// constraint that `TheInput: CanInvokeTo<TheOutput>`
///
/// For some examples of these generic functions, see the methods in
/// invoke_pools.rs and invoke_swap.rs.
pub trait CanInvokeTo<Output> {
    /// The implementation for MarginInvokeContext::invoke.
    ///
    /// Do not call this function directly. It is exposed publicly through
    /// Invoke::invoke
    fn __private_invoke(&self, inner: Instruction) -> Output;

    /// This function should return the same Output type as __private_invoke,
    /// but it should not wrap the instruction in a margin invocation
    /// instruction.
    fn __private_dont_wrap(&self, ix: Instruction) -> Output;
}

impl<K: Key> CanInvokeTo<Instruction> for MarginInvokeContext<K> {
    fn __private_invoke(&self, inner: Instruction) -> Instruction {
        self.invoke_ix(inner).0
    }
    fn __private_dont_wrap(&self, ix: Instruction) -> Instruction {
        ix
    }
}

impl CanInvokeTo<TransactionBuilder> for MarginInvokeContext<Keypair> {
    fn __private_invoke(&self, inner: Instruction) -> TransactionBuilder {
        let (wrapped, needs_signature) = self.invoke_ix(inner);
        if needs_signature {
            wrapped.with_signer(self.authority.clone())
        } else {
            wrapped.into()
        }
    }
    fn __private_dont_wrap(&self, ix: Instruction) -> TransactionBuilder {
        ix.into()
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
    pub trait InvokeEachInto<K: Key>: Sized {
        /// Only wraps the included instruction(s), without signing anything.  
        /// bool indicates if a signature is needed
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool);

        /// Separately invokes each instruction into a desired type.
        fn invoke_each_into<IxTx>(self, ctx: &MarginInvokeContext<K>) -> Vec<IxTx>
        where
            MarginInvokeContext<K>: CanInvokeTo<IxTx>,
            IxTx: From<Instruction>;
    }

    impl<K: Key> InvokeEachInto<K> for Instruction {
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
            ctx.invoke_ix(self)
        }

        fn invoke_each_into<IxTx>(self, ctx: &MarginInvokeContext<K>) -> Vec<IxTx>
        where
            MarginInvokeContext<K>: CanInvokeTo<IxTx>,
            IxTx: From<Instruction>,
        {
            vec![ctx.invoke(self)]
        }
    }

    impl<K: Key> InvokeEachInto<K> for Vec<Instruction> {
        fn invoke_ix(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
            self.into_iter()
                .map(|ix| ctx.invoke_ix(ix))
                .fold((vec![], false), |(ixs, any_need), (ix, this_needs)| {
                    (ixs.with(ix), any_need | this_needs)
                })
        }

        fn invoke_each_into<IxTx>(self, ctx: &MarginInvokeContext<K>) -> Vec<IxTx>
        where
            MarginInvokeContext<K>: CanInvokeTo<IxTx>,
            IxTx: From<Instruction>,
        {
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
        fn invoke_into(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder;
    }

    impl InvokeInto for Instruction {
        fn invoke_into(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder {
            ctx.invoke(self)
        }
    }

    impl InvokeInto for Vec<Instruction> {
        fn invoke_into(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder {
            ctx.invoke_many(self)
        }
    }
}

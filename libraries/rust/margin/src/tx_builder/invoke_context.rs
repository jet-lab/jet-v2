use jet_instructions::margin::{accounting_invoke, adapter_invoke, liquidator_invoke};
use jet_solana_client::{
    signature::{Authorization, NeedsSignature},
    transaction::{TransactionBuilder, WithSigner},
    util::{data::With, keypair::KeypairExt, Key},
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair};

/// Minimum information necessary to wrap an instruction in a margin invoke and
/// sign the transaction. Simpler alternative to MarginTxBuilder, to minimize
/// dependencies.
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
    /// Invoke a margin adapter through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer if needed.
    pub fn invoke(&self, inner: Instruction) -> TransactionBuilder {
        let (wrapped, needs_signature) = self.invoke_ix(inner);
        if needs_signature {
            wrapped.with_signer(self.authority.clone())
        } else {
            wrapped.into()
        }
    }

    /// Invoke margin adapters through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer if any instructions need it.
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

    /// Invoke margin adapters through a margin account using whatever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke.  
    /// Provides a signer for any transactions that need it.
    pub fn invoke_each(&self, ixs: Vec<Instruction>) -> Vec<TransactionBuilder> {
        ixs.into_iter().map(|ix| self.invoke(ix)).collect()
    }
}

impl MarginInvokeContext<Keypair> {
    /// conversion
    pub fn auth(&self) -> Authorization {
        Authorization {
            address: self.margin_account,
            authority: self.authority.clone(),
        }
    }
}

impl<K: Key> Clone for MarginInvokeContext<K> {
    fn clone(&self) -> Self {
        Self {
            airspace: self.airspace,
            margin_account: self.margin_account,
            authority: self.authority.clone_key(),
            is_liquidator: self.is_liquidator,
        }
    }
}

/// This trait is purely for readability, and does not introduce new behavior.
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
pub trait MarginInvoke: Sized {
    /// Only wraps the included instruction(s), without signing anything.  
    /// bool indicates if a signature is needed
    fn invoke_ix<K: Key>(self, ctx: &MarginInvokeContext<K>) -> (Self, bool);

    /// Invoke a margin adapter through a margin account using whichever wrapper
    /// is needed: adapter_invoke, accounting_invoke, or liquidator_invoke. If
    /// there are multiple instructions, they are combined into a single
    /// TransactionBuilder
    fn invoke(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder;

    /// Separately invokes each instruction into a separate TransactionBuilder
    fn invoke_each(self, ctx: &MarginInvokeContext<Keypair>) -> Vec<TransactionBuilder>;
}

impl MarginInvoke for Instruction {
    fn invoke_ix<K: Key>(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
        ctx.invoke_ix(self)
    }
    fn invoke(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder {
        ctx.invoke(self)
    }
    fn invoke_each(self, ctx: &MarginInvokeContext<Keypair>) -> Vec<TransactionBuilder> {
        vec![self.invoke(ctx)]
    }
}

impl MarginInvoke for Vec<Instruction> {
    fn invoke_ix<K: Key>(self, ctx: &MarginInvokeContext<K>) -> (Self, bool) {
        self.into_iter()
            .map(|ix| ctx.invoke_ix(ix))
            .fold((vec![], false), |(ixs, any_need), (ix, this_needs)| {
                (ixs.with(ix), any_need | this_needs)
            })
    }
    fn invoke(self, ctx: &MarginInvokeContext<Keypair>) -> TransactionBuilder {
        ctx.invoke_many(self)
    }
    fn invoke_each(self, ctx: &MarginInvokeContext<Keypair>) -> Vec<TransactionBuilder> {
        ctx.invoke_each(self)
    }
}

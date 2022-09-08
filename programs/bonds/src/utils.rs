/// Initialize a struct that has already been instantiated with invalid data.
/// Ensure safety by providing a compile-time guarantee that your code handles every field.
/// Ensure performance by not:
/// - instantiating an entire brand new struct.
/// - wasting time with fields that don't need to be changed.
macro_rules! init {
    ($zeroed_item:ident = $Struct:ident {
        $($field:ident: $value:expr),*$(,)?
    } $(ignoring {
        $($ignored_field:ident),*$(,)?
    })?) => {
        $($zeroed_item.$field = $value;)*
        #[allow(unreachable_code)]
        if false {
            // this will never run, but it enables the compiler
            // to check that every field has been mentioned.
            let _ = $Struct {
                $($field: $value,)*
                $($($ignored_field: panic!("fix the bug in `init`"),)*)?
            };
        }
    };
}
pub(crate) use init;

/// Shortcut to mint tokens in the standard case where
/// - bond_manager_authority is the mint authority
/// - all required accounts are available in a Context
///
/// There is some weirdness here with the optional parameter "bond_manager_call". If the bond
/// manager must be accessed with a method call instead of directly accessing the field, use ()
/// A clearer and more general solution would be nice but this works for now
macro_rules! mint_to {
    ($ctx:ident, $mint:ident, $recipient:ident, $amount:expr $(, $bond_manager_call:tt)?) => {
        anchor_spl::token::mint_to(
            anchor_lang::prelude::CpiContext::new(
                $ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: $ctx.accounts.$mint.to_account_info(),
                    to: $ctx.accounts.$recipient.to_account_info(),
                    authority: $ctx.accounts.bond_manager$($bond_manager_call)?.to_account_info(),
                },
            )
            .with_signer(&[&$ctx.accounts.bond_manager$($bond_manager_call)?.load()?.authority_seeds()]),
            $amount,
        )
    };
}
pub(crate) use mint_to;

/// same as above but for burning
macro_rules! burn {
    ($ctx:ident, $mint:ident, $target:ident, $amount:expr) => {
        anchor_spl::token::burn(
            anchor_lang::prelude::CpiContext::new(
                $ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: $ctx.accounts.$mint.to_account_info(),
                    from: $ctx.accounts.$target.to_account_info(),
                    authority: $ctx.accounts.bond_manager.to_account_info(),
                },
            )
            .with_signer(&[&$ctx.accounts.bond_manager.load()?.authority_seeds()]),
            $amount,
        )
    };
}
pub(crate) use burn;

/// builds context for an spl transfer invocation
macro_rules! transfer_context {
    ($ctx:ident, $to:ident, $from:ident, $authority:ident) => {
        anchor_lang::prelude::CpiContext::new(
            $ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: $ctx.accounts.$from.to_account_info(),
                to: $ctx.accounts.$to.to_account_info(),
                authority: $ctx.accounts.$authority.to_account_info(),
            },
        )
    };
}
pub(crate) use transfer_context;

/// builds accounts for an instruction on the agnostic orderbook
macro_rules! orderbook_accounts {
    ($ctx:ident, $ix:ident) => {
        agnostic_orderbook::instruction::$ix::Accounts {
            market: &$ctx.accounts.orderbook_market_state.to_account_info(),
            event_queue: &$ctx.accounts.event_queue.to_account_info(),
            bids: &$ctx.accounts.bids.to_account_info(),
            asks: &$ctx.accounts.asks.to_account_info(),
        }
    };
}
pub(crate) use orderbook_accounts;

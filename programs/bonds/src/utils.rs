use crate::control::state::BondManager;
use anchor_lang::prelude::{AccountLoader, Program};
use anchor_spl::token::Token;

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
/// derive BondTokenManager on the accounts struct to use this macro
macro_rules! mint_to {
    ($ctx:expr, $mint:ident, $recipient:ident, $amount:expr $(,)?) => {
        crate::utils::mint_to!(
            $ctx,
            $mint,
            $ctx.accounts.$recipient.to_account_info(),
            $amount
        )
    };
    ($ctx:expr, $mint:ident, $recipient:expr, $amount:expr) => {{
        use crate::utils::BondManagerProvider;
        use crate::utils::TokenProgramProvider;
        anchor_spl::token::mint_to(
            anchor_lang::prelude::CpiContext::new(
                $ctx.accounts.token_program().to_account_info(),
                anchor_spl::token::MintTo {
                    mint: $ctx.accounts.$mint.to_account_info(),
                    to: $recipient,
                    authority: $ctx.accounts.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&$ctx.accounts.bond_manager().load()?.authority_seeds()]),
            $amount,
        )
    }};
}
pub(crate) use mint_to;

/// same as above but for burning
/// burn from account owned by bond manager
/// this is used for collateral or claim notes
/// this is not used for tickets since they are owned by someone else
/// derive BondTokenManager on the accounts struct to use this macro
macro_rules! burn_notes {
    ($ctx:ident, $mint:ident, $target:ident, $amount:expr $(,)?) => {{
        use crate::utils::BondManagerProvider;
        use crate::utils::TokenProgramProvider;
        anchor_spl::token::burn(
            anchor_lang::prelude::CpiContext::new(
                $ctx.accounts.token_program().to_account_info(),
                anchor_spl::token::Burn {
                    mint: $ctx.accounts.$mint.to_account_info(),
                    from: $ctx.accounts.$target.to_account_info(),
                    authority: $ctx.accounts.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&$ctx.accounts.bond_manager().load()?.authority_seeds()]),
            $amount,
        )
    }};
}
pub(crate) use burn_notes;

/// transfer underlying tokens from vault to user
/// signed by the bond manager
/// derive BondTokenManager on the accounts struct to use this macro
macro_rules! withdraw {
    // both `from` and `to` are field names in ctx.accounts
    ($ctx:expr, $from:ident, $to:ident, $amount:expr $(,)?) => {
        crate::utils::withdraw!(
            $ctx,
            $ctx.accounts.$from.to_account_info(),
            $ctx.accounts.$to.to_account_info(),
            $amount
        )
    };
    // `from` is a field name in ctx.accounts, `to` is AccountInfo
    ($ctx:expr, $from:ident, $to:expr, $amount:expr $(, $bond_manager_nesting:ident)?) => {
        crate::utils::withdraw!($ctx, $ctx.accounts.$from.to_account_info(), $to, $amount)
    };
    // both `from` and `to` are AccountInfo
    ($ctx:expr, $from:expr, $to:expr, $amount:expr $(, $bond_manager_nesting:ident)?) => {{
        use crate::utils::BondManagerProvider;
        use crate::utils::TokenProgramProvider;
        anchor_spl::token::transfer(
            anchor_lang::prelude::CpiContext::new(
                $ctx.accounts.token_program().to_account_info(),
                anchor_spl::token::Transfer {
                    from: $from,
                    to: $to,
                    authority: $ctx.accounts.bond_manager().to_account_info(),
                },
            )
            .with_signer(&[&$ctx.accounts.bond_manager().load()?.authority_seeds()]),
            $amount,
        )
    }};
}
pub(crate) use withdraw;

/// builds accounts for an instruction on the agnostic orderbook
macro_rules! orderbook_accounts {
    ($accounts:expr, $ix:ident) => {
        agnostic_orderbook::instruction::$ix::Accounts {
            market: &$accounts.orderbook_market_state.to_account_info(),
            event_queue: &$accounts.event_queue.to_account_info(),
            bids: &$accounts.bids.to_account_info(),
            asks: &$accounts.asks.to_account_info(),
        }
    };
}
pub(crate) use orderbook_accounts;

/// Wraps an Accounts struct to use in these macros
pub struct Ctx<'a, T> {
    pub accounts: &'a T,
}

pub fn ctx<'a, T>(accounts: &'a T) -> Ctx<'a, T> {
    Ctx { accounts }
}

pub trait BondManagerProvider<'info> {
    fn bond_manager(&self) -> AccountLoader<'info, BondManager>;
}

pub trait TokenProgramProvider<'info> {
    fn token_program(&self) -> Program<'info, Token>;
}

macro_rules! map {
    ($option:ident.$($tt:tt)*) => {
        if let Some(x) = $option.as_mut() {
            x.$($tt)*
        }
    };
}
pub(crate) use map;

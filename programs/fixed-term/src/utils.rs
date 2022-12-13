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

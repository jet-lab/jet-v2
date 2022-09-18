pub use crate::{control::events::*, margin::events::*, orderbook::events::*, tickets::events::*};
use anchor_lang::{event, AnchorDeserialize, AnchorSerialize};

/// Error that was ignored because it can be handled, and it is more important to complete the instruction.
#[event]
pub struct SkippedError {
    pub message: String,
}

macro_rules! skip_err {
    ($msg:expr) => {
        emit!(crate::events::SkippedError {
            message: $msg.to_string(),
        });
        msg!($msg)
    };
    ($($arg:tt)*) => {{
		let s = format!($($arg)*);
        emit!(crate::events::SkippedError {
            message: s.to_string(),
        });
        msg!(&s)
	}};
}

pub(crate) use skip_err;

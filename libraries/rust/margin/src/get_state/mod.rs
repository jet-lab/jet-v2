/// Functions that return specific accounts from an rpc client.
mod margin_state;
/// Functions that generically return accounts from an rpc client.
mod rpc_query;

pub use margin_state::*;
pub use rpc_query::*;

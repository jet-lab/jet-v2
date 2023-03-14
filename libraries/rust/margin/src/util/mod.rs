/// simplify parallel execution of generic tasks
pub mod asynchronous;
/// non-blocking communication between threads through a queue that prevents
/// message duplication.
pub mod no_dupe_queue;

pub use jet_solana_client::util::data; // TODO: remove

/// simplify parallel execution of generic tasks
pub mod asynchronous;
/// generic processing of arbitrary data
pub mod data;
/// non-blocking communication between threads through a queue that prevents
/// message duplication.
pub mod no_dupe_queue;

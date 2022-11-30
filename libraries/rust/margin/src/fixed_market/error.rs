use thiserror::Error;

#[derive(Debug, Error)]
pub enum FixedMarketIxError {
    #[error("Missing pubkey: ({0})")]
    MissingPubkey(String),

    #[error("Failed to insert key: ({0})")]
    FailedInsert(String),

    #[error("Client error: ({msg})")]
    Client { msg: String },
}

pub type Result<T> = std::result::Result<T, FixedMarketIxError>;

pub(crate) fn client_err(err: impl ToString) -> FixedMarketIxError {
    FixedMarketIxError::Client {
        msg: err.to_string(),
    }
}

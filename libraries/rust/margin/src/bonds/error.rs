use thiserror::Error;

#[derive(Debug, Error)]
pub enum BondsIxError {
    #[error("Missing pubkey: ({0})")]
    MissingPubkey(String),

    #[error("Failed to insert key: ({0})")]
    FailedInsert(String),

    #[error("Client error: ({msg})")]
    Client { msg: String },

    #[error("Failed to deserialize event queue. Error: ({0})")]
    Deserialization(String),
}

pub type Result<T> = std::result::Result<T, BondsIxError>;

pub(crate) fn client_err(err: impl ToString) -> BondsIxError {
    BondsIxError::Client {
        msg: err.to_string(),
    }
}

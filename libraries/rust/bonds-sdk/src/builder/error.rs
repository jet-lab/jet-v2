use thiserror::Error;

#[derive(Debug, Error)]
pub enum BondsIxError {
    #[error("Missing pubkey: ({0})")]
    MissingPubkey(String),

    #[error("Failed to insert key: ({0})")]
    FailedInsert(String),

    #[cfg(feature = "utils")]
    #[error("Client error: ({msg})")]
    Client { msg: String },
}

pub type Result<T> = std::result::Result<T, BondsIxError>;

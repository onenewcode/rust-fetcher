use common::error::FetcherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IMError {
    #[error("Fetcher error: {0}")]
    Fetcher(#[from] FetcherError),

    #[error("Message error: {0}")]
    Message(String),

    #[error("API error: {0}, code: {1}")]
    Api(String, i32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protobuf error: {0}")]
    Protobuf(#[from] prost::DecodeError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IMError>;

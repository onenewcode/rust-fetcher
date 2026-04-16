use common::error::FetcherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LiveError {
    #[error("Fetcher error: {0}")]
    Fetcher(#[from] FetcherError),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Signature error: {0}")]
    Signature(String),

    #[error("Room error: {0}")]
    Room(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),

    #[error("Protobuf encode error: {0}")]
    ProtobufEncode(#[from] prost::EncodeError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, LiveError>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetcherError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JS engine error: {0}")]
    Js(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, FetcherError>;

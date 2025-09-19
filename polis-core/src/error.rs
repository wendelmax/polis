use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolisError {
    #[error("Container error: {0}")]
    Container(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Image error: {0}")]
    Image(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, PolisError>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatsError {
    #[error("System error: {0}")]
    System(String),
    
    #[error("Container not found: {0}")]
    ContainerNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, StatsError>;

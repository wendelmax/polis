use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Dockerfile error: {0}")]
    Dockerfile(String),
    
    #[error("Build context error: {0}")]
    Context(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BuildError>;

use thiserror::Error;

/// Zeaking errors
#[derive(Error, Debug)]
pub enum ZeakingError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Index not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ZeakingResult<T> = Result<T, ZeakingError>;

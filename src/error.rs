use thiserror::Error;

#[derive(Error, Debug)]
pub enum NozyError {
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type NozyResult<T> = Result<T, NozyError>;

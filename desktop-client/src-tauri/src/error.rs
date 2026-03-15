use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TauriError {
    #[error("Wallet error: {0}")]
    Wallet(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<nozy::NozyError> for TauriError {
    fn from(err: nozy::NozyError) -> Self {
        match err {
            nozy::NozyError::Storage(msg) => TauriError::Storage(msg),
            nozy::NozyError::NetworkError(msg) => TauriError::Network(msg),
            nozy::NozyError::InvalidOperation(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::Cryptographic(msg) => TauriError::Cryptographic(msg),
            nozy::NozyError::KeyDerivation(msg) => TauriError::Cryptographic(msg),
            nozy::NozyError::Transaction(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::Rpc(msg) => TauriError::Network(msg),
            nozy::NozyError::Config(msg) => TauriError::Config(msg),
            nozy::NozyError::AddressParsing(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::NoteCommitment(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::MerklePath(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::BundleAuthorization(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::NoteScanning(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::InsufficientFunds(msg) => TauriError::InvalidOperation(msg),
            nozy::NozyError::InvalidInput(msg) => TauriError::InvalidOperation(msg),
        }
    }
}

pub type TauriResult<T> = Result<T, TauriError>;

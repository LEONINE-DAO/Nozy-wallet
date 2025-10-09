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
    
    #[error("Address parsing error: {0}")]
    AddressParsing(String),
    
    #[error("Note commitment error: {0}")]
    NoteCommitment(String),
    
    #[error("Merkle path error: {0}")]
    MerklePath(String),
    
    #[error("Bundle authorization error: {0}")]
    BundleAuthorization(String),
    
    #[error("Note scanning error: {0}")]
    NoteScanning(String),
    
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),
}

impl NozyError {
    /// Create a user-friendly error message with context
    pub fn with_context(self, context: &str) -> Self {
        match self {
            NozyError::KeyDerivation(msg) => NozyError::KeyDerivation(format!("{}: {}", context, msg)),
            NozyError::Storage(msg) => NozyError::Storage(format!("{}: {}", context, msg)),
            NozyError::NetworkError(msg) => NozyError::NetworkError(format!("{}: {}", context, msg)),
            NozyError::InvalidOperation(msg) => NozyError::InvalidOperation(format!("{}: {}", context, msg)),
            NozyError::Transaction(msg) => NozyError::Transaction(format!("{}: {}", context, msg)),
            NozyError::Config(msg) => NozyError::Config(format!("{}: {}", context, msg)),
            NozyError::AddressParsing(msg) => NozyError::AddressParsing(format!("{}: {}", context, msg)),
            NozyError::NoteCommitment(msg) => NozyError::NoteCommitment(format!("{}: {}", context, msg)),
            NozyError::MerklePath(msg) => NozyError::MerklePath(format!("{}: {}", context, msg)),
            NozyError::BundleAuthorization(msg) => NozyError::BundleAuthorization(format!("{}: {}", context, msg)),
            NozyError::NoteScanning(msg) => NozyError::NoteScanning(format!("{}: {}", context, msg)),
            NozyError::Rpc(msg) => NozyError::Rpc(format!("{}: {}", context, msg)),
            NozyError::Cryptographic(msg) => NozyError::Cryptographic(format!("{}: {}", context, msg)),
        }
    }
    
    /// Get a user-friendly error message with suggestions
    pub fn user_friendly_message(&self) -> String {
        match self {
            NozyError::NetworkError(_) => {
                "Network connection failed. Please check your Zebra node connection and try again.".to_string()
            },
            NozyError::AddressParsing(_) => {
                "Invalid address format. Please ensure you're using a valid Zcash unified address.".to_string()
            },
            NozyError::Transaction(_) => {
                "Transaction failed. Please check your inputs and try again.".to_string()
            },
            NozyError::KeyDerivation(_) => {
                "Key derivation failed. Please check your wallet seed and try again.".to_string()
            },
            NozyError::Storage(_) => {
                "Storage error. Please check your wallet file permissions and try again.".to_string()
            },
            _ => format!("An error occurred: {}", self)
        }
    }
}

pub type NozyResult<T> = Result<T, NozyError>;

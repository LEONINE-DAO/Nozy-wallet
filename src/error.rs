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
    
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl NozyError {
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
            NozyError::InsufficientFunds(msg) => NozyError::InsufficientFunds(format!("{}: {}", context, msg)),
            NozyError::InvalidInput(msg) => NozyError::InvalidInput(format!("{}: {}", context, msg)),
        }
    }
    
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
            NozyError::InsufficientFunds(_) => {
                "Insufficient funds. Please check your balance and try again.".to_string()
            },
            NozyError::InvalidInput(_) => {
                "Invalid input. Please check your input values and try again.".to_string()
            },
            _ => format!("An error occurred: {}", self)
        }
    }
    
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            NozyError::NetworkError(_) => vec![
                "Check if Zebra node is running: 'zebrad start'".to_string(),
                "Verify Zebra RPC URL in config: 'nozy config'".to_string(),
                "Check network connectivity".to_string(),
                "Try restarting Zebra node".to_string(),
            ],
            NozyError::InsufficientFunds(_) => vec![
                "Check your balance: 'nozy balance'".to_string(),
                "Wait for pending transactions to confirm".to_string(),
                "Sync your wallet: 'nozy sync'".to_string(),
            ],
            NozyError::AddressParsing(_) => vec![
                "Ensure address starts with 'u1' (unified address)".to_string(),
                "Verify address is for the correct network (mainnet/testnet)".to_string(),
                "Check for typos in the address".to_string(),
            ],
            NozyError::Transaction(_) => vec![
                "Verify recipient address is correct".to_string(),
                "Check transaction amount is valid".to_string(),
                "Ensure sufficient funds including fees".to_string(),
                "Try again after a few moments".to_string(),
            ],
            NozyError::Storage(_) => vec![
                "Check wallet file permissions".to_string(),
                "Ensure sufficient disk space".to_string(),
                "Verify wallet directory is writable".to_string(),
            ],
            NozyError::KeyDerivation(_) => vec![
                "Verify mnemonic phrase is correct".to_string(),
                "Check wallet seed is valid".to_string(),
                "Try restoring wallet from mnemonic".to_string(),
            ],
            _ => vec![
                "Check the error message above for details".to_string(),
                "Review your input values".to_string(),
                "Try the operation again".to_string(),
            ],
        }
    }
}

pub type NozyResult<T> = Result<T, NozyError>;

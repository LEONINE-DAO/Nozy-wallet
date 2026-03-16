use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TauriError {
    pub message: String,
    pub code: Option<String>,
}

impl From<nozy::NozyError> for TauriError {
    fn from(err: nozy::NozyError) -> Self {
        let code = match &err {
            nozy::NozyError::Storage(_) => Some("WALLET_NOT_FOUND".to_string()),
            nozy::NozyError::InvalidInput(_) => Some("INVALID_PASSWORD".to_string()),
            nozy::NozyError::InsufficientFunds(_) => Some("INSUFFICIENT_FUNDS".to_string()),
            nozy::NozyError::NetworkError(_) => Some("NETWORK_ERROR".to_string()),
            nozy::NozyError::Rpc(_) => Some("RPC_ERROR".to_string()),
            nozy::NozyError::AddressParsing(_) => Some("INVALID_ADDRESS".to_string()),
            nozy::NozyError::Transaction(_) => Some("TRANSACTION_ERROR".to_string()),
            _ => None,
        };
        
        TauriError {
            message: err.user_friendly_message(),
            code,
        }
    }
}

impl From<String> for TauriError {
    fn from(s: String) -> Self {
        TauriError {
            message: s,
            code: None,
        }
    }
}

impl From<&str> for TauriError {
    fn from(s: &str) -> Self {
        TauriError {
            message: s.to_string(),
            code: None,
        }
    }
}


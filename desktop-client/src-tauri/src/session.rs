use crate::error::TauriError;
use nozy::{HDWallet, WalletStorage};
use std::sync::Mutex;

static UNLOCK_PASSWORD: Mutex<Option<String>> = Mutex::new(None);

pub fn set_unlock_password(password: String) {
    if let Ok(mut guard) = UNLOCK_PASSWORD.lock() {
        *guard = Some(password);
    }
}

pub fn clear_session() {
    if let Ok(mut guard) = UNLOCK_PASSWORD.lock() {
        *guard = None;
    }
}

pub fn is_session_unlocked() -> bool {
    UNLOCK_PASSWORD
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

pub fn session_password() -> Option<String> {
    UNLOCK_PASSWORD.lock().ok().and_then(|guard| guard.clone())
}

pub fn resolve_password(override_password: Option<&str>) -> String {
    let from_request = override_password
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string);
    from_request
        .or_else(session_password)
        .unwrap_or_default()
}

pub async fn load_session_wallet(
    override_password: Option<&str>,
) -> Result<HDWallet, TauriError> {
    let password = resolve_password(override_password);
    let storage = WalletStorage::with_xdg_dir();
    storage.load_wallet(&password).await.map_err(|e| {
        let msg = e.to_string();
        let code = if msg.contains("Invalid password")
            || msg.contains("Decryption failed")
        {
            "INVALID_PASSWORD"
        } else if !is_session_unlocked() && password.is_empty() {
            "WALLET_LOCKED"
        } else {
            "WALLET_NOT_FOUND"
        };
        let message = if code == "INVALID_PASSWORD" {
            "Incorrect password. Use the same password as Welcome → Unlock (or leave blank if already unlocked)."
                .to_string()
        } else if code == "WALLET_LOCKED" {
            "Wallet is locked. Unlock on the Welcome screen or enter your password.".to_string()
        } else {
            format!("Failed to load wallet: {e}")
        };
        TauriError {
            message,
            code: Some(code.to_string()),
        }
    })
}

pub fn verify_wallet_password(wallet: &HDWallet, password: &str) -> Result<(), TauriError> {
    if !wallet.is_password_protected() {
        return Ok(());
    }
    let valid = wallet
        .verify_password(password)
        .map_err(|e| TauriError::from(e.to_string()))?;
    if !valid {
        return Err(TauriError {
            message: "Invalid password".to_string(),
            code: Some("INVALID_PASSWORD".to_string()),
        });
    }
    Ok(())
}

/// Load wallet for mnemonic / private-key reveal. No-password wallets may use an empty
/// request password; password-protected wallets must supply the password explicitly.
pub async fn load_wallet_for_reveal(request_password: &str) -> Result<HDWallet, TauriError> {
    let override_pw = if request_password.is_empty() {
        None
    } else {
        Some(request_password)
    };
    let wallet = load_session_wallet(override_pw).await?;

    if wallet.is_password_protected() {
        if request_password.is_empty() {
            return Err(TauriError {
                message: "Please enter your wallet password".to_string(),
                code: Some("AUTH_002".to_string()),
            });
        }
        verify_wallet_password(&wallet, request_password)?;
    }

    Ok(wallet)
}

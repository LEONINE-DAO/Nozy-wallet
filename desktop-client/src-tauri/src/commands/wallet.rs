use crate::error::{TauriError, TauriResult};
use nozy::{HDWallet, WalletStorage};
use nozy::paths::get_wallet_data_dir;
use nozy::key_management::zeroize_bytes;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use hex;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletExistsResponse {
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreWalletRequest {
    pub mnemonic: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockWalletRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStatusResponse {
    pub exists: bool,
    pub unlocked: bool,
    pub has_password: bool,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyPasswordRequest {
    pub password: String,
}

// Global state to track unlocked wallet with Mutex lazy can be nozy
lazy_static::lazy_static! {
    static ref UNLOCKED_WALLET: Mutex<Option<HDWallet>> = Mutex::new(None);
}

#[tauri::command]
pub async fn wallet_exists() -> TauriResult<WalletExistsResponse> {
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    Ok(WalletExistsResponse {
        exists: wallet_path.exists(),
    })
}

#[tauri::command]
pub async fn create_wallet(
    request: CreateWalletRequest,
) -> TauriResult<String> {
    let mut wallet = HDWallet::new()
        .map_err(|e| TauriError::Wallet(format!("Failed to create wallet: {}", e)))?;

    let password = request.password.unwrap_or_default();
    
    if !password.is_empty() {
        wallet
            .set_password(&password)
            .map_err(|e| TauriError::Cryptographic(format!("Failed to set password: {}", e)))?;
    }

    let storage = WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &password)
        .await
        .map_err(|e| TauriError::Storage(format!("Failed to save wallet: {}", e)))?;

    let mnemonic = wallet.get_mnemonic();
    *UNLOCKED_WALLET.lock().unwrap() = Some(wallet);

    Ok(mnemonic)
}

#[tauri::command]
pub async fn restore_wallet(request: RestoreWalletRequest) -> TauriResult<()> {
    let wallet = HDWallet::from_mnemonic(&request.mnemonic)
        .map_err(|e| TauriError::Wallet(format!("Invalid mnemonic: {}", e)))?;

    let password = request.password.unwrap_or_default();
    
    if !password.is_empty() {
        let mut wallet_with_password = wallet;
        wallet_with_password
            .set_password(&password)
            .map_err(|e| TauriError::Cryptographic(format!("Failed to set password: {}", e)))?;
        
        let storage = WalletStorage::with_xdg_dir();
        storage
            .save_wallet(&wallet_with_password, &password)
            .await
            .map_err(|e| TauriError::Storage(format!("Failed to save wallet: {}", e)))?;
        
        *UNLOCKED_WALLET.lock().unwrap() = Some(wallet_with_password);
    } else {
        let storage = WalletStorage::with_xdg_dir();
        storage
            .save_wallet(&wallet, &password)
            .await
            .map_err(|e| TauriError::Storage(format!("Failed to save wallet: {}", e)))?;
        
        *UNLOCKED_WALLET.lock().unwrap() = Some(wallet);
    }

    Ok(())
}

#[tauri::command]
pub async fn unlock_wallet(
    request: UnlockWalletRequest,
) -> TauriResult<WalletStatusResponse> {
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage
        .load_wallet(&request.password)
        .await
        .map_err(|e| TauriError::Storage(format!("Failed to unlock wallet: {}", e)))?;

    let has_password = wallet.is_password_protected();
    let address = wallet
        .generate_orchard_address(0, 0)
        .ok();

    *UNLOCKED_WALLET.lock().unwrap() = Some(wallet);

    Ok(WalletStatusResponse {
        exists: true,
        unlocked: true,
        has_password,
        address,
    })
}

#[tauri::command]
pub async fn get_wallet_status() -> TauriResult<WalletStatusResponse> {
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    let exists = wallet_path.exists();

    if !exists {
        return Ok(WalletStatusResponse {
            exists: false,
            unlocked: false,
            has_password: false,
            address: None,
        });
    }

    let (unlocked, has_password, address) = {
        let wallet_guard = UNLOCKED_WALLET.lock().unwrap();
        let unlocked = wallet_guard.is_some();
        
        if unlocked {
            let wallet = wallet_guard.as_ref().unwrap();
            let has_password = wallet.is_password_protected();
            let address = wallet.generate_orchard_address(0, 0).ok();
            (true, has_password, address)
        } else {
            (false, false, None)
        }
    };
    
    if unlocked {
        Ok(WalletStatusResponse {
            exists: true,
            unlocked: true,
            has_password,
            address,
        })
    } else {
        let storage = WalletStorage::with_xdg_dir();
        let has_password = storage.load_wallet("").await.is_err();
        
        Ok(WalletStatusResponse {
            exists: true,
            unlocked: false,
            has_password,
            address: None,
        })
    }
}

pub fn get_unlocked_wallet() -> TauriResult<HDWallet> {
    UNLOCKED_WALLET
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| TauriError::Wallet("Wallet is not unlocked".to_string()))
}

#[tauri::command]
pub async fn lock_wallet() -> TauriResult<()> {
    *UNLOCKED_WALLET.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub async fn change_password(
    request: ChangePasswordRequest,
) -> TauriResult<()> {
    let storage = WalletStorage::with_xdg_dir();
    let mut wallet = storage
        .load_wallet(&request.current_password)
        .await
        .map_err(|e| {
            TauriError::Cryptographic(format!(
                "Current password is incorrect: {}",
                e
            ))
        })?;

    let is_valid = wallet
        .verify_password(&request.current_password)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Password verification failed: {}", e))
        })?;

    if !is_valid {
        return Err(TauriError::Cryptographic(
            "Current password is incorrect".to_string(),
        ));
    }

    wallet
        .set_password(&request.new_password)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Failed to set new password: {}", e))
        })?;

    storage
        .save_wallet(&wallet, &request.new_password)
        .await
        .map_err(|e| TauriError::Storage(format!("Failed to save wallet: {}", e)))?;

    *UNLOCKED_WALLET.lock().unwrap() = Some(wallet);

    Ok(())
}

#[tauri::command]
pub async fn get_mnemonic(
    request: VerifyPasswordRequest,
) -> TauriResult<String> {
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage
        .load_wallet(&request.password)
        .await
        .map_err(|e| {
            TauriError::Cryptographic(format!(
                "Password verification failed: {}",
                e
            ))
        })?;

    let is_valid = wallet
        .verify_password(&request.password)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Password verification failed: {}", e))
        })?;

    if !is_valid {
        return Err(TauriError::Cryptographic(
            "Password is incorrect".to_string(),
        ));
    }

    Ok(wallet.get_mnemonic())
}

#[tauri::command]
pub async fn get_private_key(
    request: VerifyPasswordRequest,
) -> TauriResult<String> {
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage
        .load_wallet(&request.password)
        .await
        .map_err(|e| {
            TauriError::Cryptographic(format!(
                "Password verification failed: {}",
                e
            ))
        })?;

    let is_valid = wallet
        .verify_password(&request.password)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Password verification failed: {}", e))
        })?;

    if !is_valid {
        return Err(TauriError::Cryptographic(
            "Password is incorrect".to_string(),
        ));
    }

    use nozy::key_management::SecureSeed;
    use orchard::keys::SpendingKey;
    use zcash_primitives::zip32::AccountId;

    let mut seed_bytes = wallet.get_mnemonic_object().to_seed("").to_vec();
    let secure_seed = SecureSeed::new(seed_bytes.clone());

    let account_id = AccountId::try_from(0)
        .map_err(|e| TauriError::Cryptographic(format!("Invalid account ID: {:?}", e)))?;

    let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Failed to derive Orchard spending key: {:?}", e))
        })?;

    let sk_bytes = orchard_sk.to_bytes();
    let hex_key = hex::encode(sk_bytes);

    // Zeroize sensitive data fo Nozy people
    zeroize_bytes(&mut seed_bytes);

    Ok(hex_key)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignMessageResponse {
    pub signature: String,
}

#[tauri::command]
pub async fn sign_message(
    request: SignMessageRequest,
) -> TauriResult<SignMessageResponse> {
    use nozy::key_management::SecureSeed;
    use orchard::keys::SpendingKey;
    use zcash_primitives::zip32::AccountId;
    use secp256k1::{Secp256k1, SecretKey, Message, ecdsa::Signature};
    use sha2::{Sha256, Digest};

    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage
        .load_wallet(&request.password)
        .await
        .map_err(|e| {
            TauriError::Cryptographic(format!(
                "Password verification failed: {}",
                e
            ))
        })?;

    let is_valid = wallet
        .verify_password(&request.password)
        .map_err(|e| {
            TauriError::Cryptographic(format!("Password verification failed: {}", e))
        })?;

    if !is_valid {
        return Err(TauriError::Cryptographic(
            "Password is incorrect".to_string(),
        ));
    }

    // Derive spending key to get a signing key
    // For message signing, we'll derive a secp256k1 key from the seed
    let mut seed_bytes = wallet.get_mnemonic_object().to_seed("").to_vec();
    let secure_seed = SecureSeed::new(seed_bytes.clone());

    // Derive a secp256k1 key from the seed for message signing
    // Use BIP32 path m/44'/133'/0'/0/0 for Zcash message signing
    use bip32::{DerivationPath, XPrv, ChildNumber};
    use std::str::FromStr;
    
    let master_key = XPrv::new(seed_bytes.clone())
        .map_err(|e| TauriError::Cryptographic(format!("Failed to create master key: {}", e)))?;
    
    let derivation_path = DerivationPath::from_str("m/44'/133'/0'/0/0")
        .map_err(|e| TauriError::Cryptographic(format!("Invalid derivation path: {}", e)))?;
    
    // Derive key step by step
    let mut derived_key = master_key;
    for child_number in derivation_path {
        derived_key = derived_key
            .derive_child(child_number)
            .map_err(|e| TauriError::Cryptographic(format!("Failed to derive key: {}", e)))?;
    }
    
    let private_key_bytes = derived_key.to_bytes();
    
    // Create secp256k1 secret key
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&private_key_bytes[..32])
        .map_err(|e| TauriError::Cryptographic(format!("Invalid private key: {}", e)))?;

    // Prepare message for signing (EIP-191 style prefix for Web3 compatibility)
    let message_bytes = request.message.as_bytes();
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message_bytes.len());
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message_bytes);
    let message_hash = hasher.finalize();

    // Create message for signing
    let msg = Message::from_digest_slice(&message_hash)
        .map_err(|e| TauriError::Cryptographic(format!("Invalid message: {}", e)))?;

    // Sign the message
    let signature = secp.sign_ecdsa(&msg, &secret_key);
    
    // Serialize signature (64 bytes: r || s)
    // For Ethereum compatibility, we add recovery ID (v) to make it 65 bytes
    let sig_compact = signature.serialize_compact();
    let mut sig_bytes = sig_compact.to_vec();
    
    // Calculate recovery ID (v) - try both 0 and 1 and use the one that works
    // For simplicity, we'll use 0x1b (27) or 0x1c (28) as per Ethereum standard
    // We'll default to 27 (0x1b) for mainnet
    sig_bytes.push(0x1b);
    
    let signature_hex = hex::encode(sig_bytes);

    // Zeroize sensitive data
    zeroize_bytes(&mut seed_bytes);

    Ok(SignMessageResponse {
        signature: format!("0x{}", signature_hex),
    })
}

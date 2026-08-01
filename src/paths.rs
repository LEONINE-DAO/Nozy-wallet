use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Optional process-wide wallet data dir override (used by `nozy-ffi` on mobile).
static WALLET_DATA_DIR_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Run `f` with [`get_wallet_data_dir`] resolving to `dir` (notes, compact DB, etc.).
pub fn with_wallet_data_dir<T>(dir: impl AsRef<Path>, f: impl FnOnce() -> T) -> T {
    let path = dir.as_ref().to_path_buf();
    let _ = std::fs::create_dir_all(&path);
    let previous = {
        let mut guard = WALLET_DATA_DIR_OVERRIDE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.replace(path)
    };
    let result = f();
    let mut guard = WALLET_DATA_DIR_OVERRIDE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = previous;
    result
}

/// Root Nozy data directory (profiles manifest and per-wallet subdirectories).
pub fn get_wallet_base_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "nozy", "nozy") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        data_dir.to_path_buf()
    } else {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let fallback = PathBuf::from(&home_dir).join(".nozy").join("data");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    }
}

/// Active wallet profile data directory (wallet.dat, notes, sync DB, etc.).
pub fn get_wallet_data_dir() -> PathBuf {
    if let Ok(guard) = WALLET_DATA_DIR_OVERRIDE.lock() {
        if let Some(ref dir) = *guard {
            return dir.clone();
        }
    }
    crate::wallet_profiles::active_profile_data_dir()
}

pub fn get_wallet_config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "nozy", "nozy") {
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir).ok();
        config_dir.to_path_buf()
    } else {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let fallback = PathBuf::from(&home_dir).join(".nozy").join("config");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    }
}

pub fn get_wallet_data_path() -> PathBuf {
    get_wallet_data_dir()
}

pub fn get_wallet_config_path() -> PathBuf {
    get_wallet_config_dir().join("config.json")
}

/// Get the Zeaking index directory path
pub fn get_zeaking_index_dir() -> PathBuf {
    get_wallet_data_dir().join("zeaking")
}

//! Multiple wallet profiles — each profile has its own data directory under
//! `{base}/profiles/{id}/` (wallet.dat, notes, sync DB, etc.).

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_base_dir;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

const MANIFEST_VERSION: u32 = 1;

static PROFILES_INIT: Once = Once::new();

const PROFILE_SCOPED_FILES: &[&str] = &[
    "wallet.dat",
    "notes.json",
    "lwd_compact.sqlite",
    "analytics.json",
    "keystone_pending_send.json",
    "address_book.json",
    "sent_transactions.json",
    "wallet_current_backup.dat",
];

const PROFILE_SCOPED_DIRS: &[&str] = &["orchard_params", "zeaking"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletProfile {
    pub id: String,
    pub name: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesManifest {
    version: u32,
    #[serde(default)]
    active_id: Option<String>,
    #[serde(default)]
    profiles: Vec<WalletProfile>,
}

impl Default for ProfilesManifest {
    fn default() -> Self {
        Self {
            version: MANIFEST_VERSION,
            active_id: None,
            profiles: Vec::new(),
        }
    }
}

fn profiles_root(base: &Path) -> PathBuf {
    base.join("profiles")
}

fn manifest_path(base: &Path) -> PathBuf {
    profiles_root(base).join("manifest.json")
}

fn profile_dir(base: &Path, id: &str) -> PathBuf {
    profiles_root(base).join(id)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn new_profile_id() -> String {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    format!("{:016x}", u64::from_be_bytes(bytes))
}

fn default_profile_name(count: usize) -> String {
    format!("Wallet {}", count + 1)
}

fn move_path(src: &Path, dest: &Path) -> NozyResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            NozyError::Storage(format!("Failed to create profile directory: {}", e))
        })?;
    }
    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            if src.is_dir() {
                copy_dir_recursive(src, dest)?;
                fs::remove_dir_all(src).ok();
            } else {
                fs::copy(src, dest).map_err(|e| {
                    NozyError::Storage(format!("Failed to copy {}: {}", src.display(), e))
                })?;
                fs::remove_file(src).ok();
            }
            Ok(())
        }
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> NozyResult<()> {
    fs::create_dir_all(dest).map_err(|e| {
        NozyError::Storage(format!(
            "Failed to create directory {}: {}",
            dest.display(),
            e
        ))
    })?;
    for entry in fs::read_dir(src).map_err(|e| {
        NozyError::Storage(format!("Failed to read directory {}: {}", src.display(), e))
    })? {
        let entry = entry.map_err(|e| NozyError::Storage(e.to_string()))?;
        let target = dest.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| {
                NozyError::Storage(format!("Failed to copy {}: {}", entry.path().display(), e))
            })?;
        }
    }
    Ok(())
}

fn migrate_legacy_wallet_to_profile(base: &Path, dest_dir: &Path) -> NozyResult<()> {
    fs::create_dir_all(dest_dir)
        .map_err(|e| NozyError::Storage(format!("Failed to create profile directory: {}", e)))?;

    for file in PROFILE_SCOPED_FILES {
        let src = base.join(file);
        if src.exists() {
            move_path(&src, &dest_dir.join(file))?;
        }
    }

    for dir in PROFILE_SCOPED_DIRS {
        let src = base.join(dir);
        if src.is_dir() {
            move_path(&src, &dest_dir.join(dir))?;
        }
    }

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("wallet_backup_") {
                move_path(&entry.path(), &dest_dir.join(&name))?;
            }
        }
    }

    Ok(())
}

fn load_manifest(base: &Path) -> NozyResult<ProfilesManifest> {
    let path = manifest_path(base);
    if !path.exists() {
        return Ok(ProfilesManifest::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| NozyError::Storage(format!("Failed to read wallet profiles: {}", e)))?;
    serde_json::from_str(&raw)
        .map_err(|e| NozyError::Storage(format!("Failed to parse wallet profiles: {}", e)))
}

fn save_manifest(base: &Path, manifest: &ProfilesManifest) -> NozyResult<()> {
    let root = profiles_root(base);
    fs::create_dir_all(&root)
        .map_err(|e| NozyError::Storage(format!("Failed to create profiles directory: {}", e)))?;
    let serialized = serde_json::to_string_pretty(manifest)
        .map_err(|e| NozyError::Storage(format!("Failed to serialize wallet profiles: {}", e)))?;
    fs::write(manifest_path(base), serialized)
        .map_err(|e| NozyError::Storage(format!("Failed to write wallet profiles: {}", e)))
}

pub fn ensure_profiles_initialized() -> NozyResult<()> {
    let base = get_wallet_base_dir();
    if manifest_path(&base).exists() {
        return Ok(());
    }

    if base.join("wallet.dat").exists() {
        let profile = WalletProfile {
            id: new_profile_id(),
            name: default_profile_name(0),
            created_at: now_secs(),
        };
        let dest = profile_dir(&base, &profile.id);
        migrate_legacy_wallet_to_profile(&base, &dest)?;

        let manifest = ProfilesManifest {
            version: MANIFEST_VERSION,
            active_id: Some(profile.id.clone()),
            profiles: vec![profile],
        };
        save_manifest(&base, &manifest)?;
        return Ok(());
    }

    save_manifest(&base, &ProfilesManifest::default())
}

fn ensure_initialized_once() {
    PROFILES_INIT.call_once(|| {
        if let Err(e) = ensure_profiles_initialized() {
            eprintln!("Warning: failed to initialize wallet profiles: {}", e);
        }
        if let Err(e) = migrate_orphaned_sent_transactions() {
            eprintln!("Warning: failed to migrate sent transaction history: {}", e);
        }
    });
}

/// Copy legacy `sent_transactions.json` from the base data dir into wallet profile folders.
/// Sends made before multi-wallet profiles were stored at the base path and are invisible
/// to the active profile until migrated.
pub fn migrate_orphaned_sent_transactions() -> NozyResult<()> {
    let base = get_wallet_base_dir();
    let legacy = base.join("sent_transactions.json");
    if !legacy.exists() {
        return Ok(());
    }

    let manifest = load_manifest(&base)?;
    let mut target_ids = std::collections::HashSet::new();
    if let Some(active_id) = manifest.active_id {
        target_ids.insert(active_id);
    }
    if let Some(oldest) = manifest
        .profiles
        .iter()
        .min_by_key(|profile| profile.created_at)
        .map(|profile| profile.id.clone())
    {
        target_ids.insert(oldest);
    }

    for profile_id in target_ids {
        let dest_dir = profile_dir(&base, &profile_id);
        let dest = dest_dir.join("sent_transactions.json");
        if dest.exists() {
            continue;
        }
        fs::create_dir_all(&dest_dir).map_err(|e| {
            NozyError::Storage(format!("Failed to create wallet profile directory: {}", e))
        })?;
        fs::copy(&legacy, &dest).map_err(|e| {
            NozyError::Storage(format!(
                "Failed to migrate sent transaction history to {}: {}",
                dest.display(),
                e
            ))
        })?;
    }

    Ok(())
}

pub fn active_profile_id() -> Option<String> {
    ensure_initialized_once();
    let base = get_wallet_base_dir();
    load_manifest(&base).ok().and_then(|m| m.active_id)
}

/// Active profile data directory (wallet.dat, notes, sync DB, etc.).
pub fn active_profile_data_dir() -> PathBuf {
    ensure_initialized_once();
    let base = get_wallet_base_dir();
    let manifest = load_manifest(&base).unwrap_or_default();
    if let Some(active_id) = manifest.active_id {
        return profile_dir(&base, &active_id);
    }
    profiles_root(&base).join("_inactive")
}

pub fn active_wallet_exists() -> bool {
    ensure_initialized_once();
    active_profile_data_dir().join("wallet.dat").exists()
}

pub fn list_wallet_profiles() -> NozyResult<Vec<WalletProfile>> {
    ensure_initialized_once();
    let base = get_wallet_base_dir();
    Ok(load_manifest(&base)?.profiles)
}

pub fn profile_has_wallet(id: &str) -> bool {
    ensure_initialized_once();
    profile_dir(&get_wallet_base_dir(), id)
        .join("wallet.dat")
        .exists()
}

/// Create a new empty profile and make it active. Existing profiles are preserved.
pub fn create_new_profile(name: Option<&str>) -> NozyResult<WalletProfile> {
    ensure_initialized_once();
    let base = get_wallet_base_dir();
    let mut manifest = load_manifest(&base)?;

    let profile = WalletProfile {
        id: new_profile_id(),
        name: name
            .map(str::to_string)
            .unwrap_or_else(|| default_profile_name(manifest.profiles.len())),
        created_at: now_secs(),
    };

    fs::create_dir_all(profile_dir(&base, &profile.id)).map_err(|e| {
        NozyError::Storage(format!("Failed to create wallet profile directory: {}", e))
    })?;

    manifest.profiles.push(profile.clone());
    manifest.active_id = Some(profile.id.clone());
    save_manifest(&base, &manifest)?;
    Ok(profile)
}

pub fn set_active_wallet_profile(id: &str) -> NozyResult<()> {
    ensure_initialized_once();
    let base = get_wallet_base_dir();
    let mut manifest = load_manifest(&base)?;
    if !manifest.profiles.iter().any(|p| p.id == id) {
        return Err(NozyError::Storage(format!(
            "Wallet profile not found: {}",
            id
        )));
    }
    manifest.active_id = Some(id.to_string());
    save_manifest(&base, &manifest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_base_dir() -> PathBuf {
        let mut dir = env::temp_dir();
        dir.push(format!("nozy-profiles-test-{}", new_profile_id()));
        dir
    }

    #[test]
    fn create_multiple_profiles_preserves_existing() {
        let base = temp_base_dir();
        let _ = fs::remove_dir_all(&base);

        let profile1 = {
            let mut manifest = ProfilesManifest::default();
            let p = WalletProfile {
                id: new_profile_id(),
                name: "Wallet 1".to_string(),
                created_at: now_secs(),
            };
            fs::create_dir_all(profile_dir(&base, &p.id)).unwrap();
            fs::write(profile_dir(&base, &p.id).join("wallet.dat"), b"wallet-1").unwrap();
            manifest.profiles.push(p.clone());
            manifest.active_id = Some(p.id.clone());
            save_manifest(&base, &manifest).unwrap();
            p
        };

        let mut manifest = load_manifest(&base).unwrap();
        let profile2 = WalletProfile {
            id: new_profile_id(),
            name: "Wallet 2".to_string(),
            created_at: now_secs(),
        };
        fs::create_dir_all(profile_dir(&base, &profile2.id)).unwrap();
        manifest.profiles.push(profile2.clone());
        manifest.active_id = Some(profile2.id.clone());
        save_manifest(&base, &manifest).unwrap();

        assert!(profile_dir(&base, &profile1.id).join("wallet.dat").exists());
        assert!(
            profile_dir(&base, &profile2.id)
                .join("wallet.dat")
                .is_file()
                == false
        );

        let _ = fs::remove_dir_all(&base);
    }
}

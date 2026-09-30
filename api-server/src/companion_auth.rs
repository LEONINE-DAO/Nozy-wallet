//! Resolve the companion API key used to protect fund-moving and seed routes.

use std::fs;
use std::path::PathBuf;

use tracing::{info, warn};

const KEY_FILE_NAME: &str = "companion_api_key";

fn key_file_path() -> PathBuf {
    nozy::paths::get_wallet_data_dir().join(KEY_FILE_NAME)
}

fn generate_api_key() -> anyhow::Result<String> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf)
        .map_err(|e| anyhow::anyhow!("failed to generate companion API key: {e}"))?;
    Ok(hex::encode(buf))
}

/// Resolve API key for authenticated routes.
///
/// Order: `NOZY_API_KEY` env → `{wallet_data}/companion_api_key` → generate + persist.
/// Set `NOZY_ALLOW_UNAUTHENTICATED=1` only for emergency/dev (logs a warning).
pub fn resolve_companion_api_key() -> anyhow::Result<Option<String>> {
    if std::env::var("NOZY_ALLOW_UNAUTHENTICATED").is_ok() {
        warn!(
            "NOZY_ALLOW_UNAUTHENTICATED is set — fund/seed HTTP routes are open on the bind address. \
             Do not use this with real funds."
        );
        return Ok(None);
    }

    if let Ok(env_key) = std::env::var("NOZY_API_KEY") {
        let trimmed = env_key.trim();
        if !trimmed.is_empty() {
            info!("Using NOZY_API_KEY from environment");
            return Ok(Some(trimmed.to_string()));
        }
    }

    let path = key_file_path();
    if path.exists() {
        let contents = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        let trimmed = contents.trim();
        if !trimmed.is_empty() {
            info!("Using companion API key from {}", path.display());
            return Ok(Some(trimmed.to_string()));
        }
    }

    let key = generate_api_key()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("failed to create {}: {e}", parent.display()))?;
    }
    fs::write(&path, format!("{key}\n"))
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    info!(
        "Generated companion API key at {} — paste into the extension Companion settings (X-API-Key)",
        path.display()
    );
    Ok(Some(key))
}

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

/// Lexically normalize a path (resolve `.` / `..` without requiring the target to exist).
fn normalize_path_components(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                let _ = out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn path_allowlist_roots() -> Vec<PathBuf> {
    let mut roots = vec![get_wallet_base_dir(), get_wallet_data_dir()];
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let home = PathBuf::from(home);
        roots.push(home.clone());
        roots.push(home.join("Documents"));
        roots.push(home.join("Desktop"));
        roots.push(home.join("Downloads"));
    }
    roots
}

fn path_is_under_root(path: &Path, root: &Path) -> bool {
    let path_n = normalize_path_components(path);
    let root_n = normalize_path_components(root);
    path_n.starts_with(&root_n)
}

/// Resolve a user-supplied path and require it under wallet data / home allowlist (F-11).
pub fn resolve_allowlisted_user_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Path is required.".to_string());
    }
    if trimmed.contains('\0') {
        return Err("Path contains invalid characters.".to_string());
    }

    let candidate = PathBuf::from(trimmed);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Cannot resolve relative path: {e}"))?
            .join(candidate)
    };
    let normalized = normalize_path_components(&absolute);

    for root in path_allowlist_roots() {
        if path_is_under_root(&normalized, &root) {
            return Ok(normalized);
        }
        // Also accept when an existing root canonicalizes (symlinks / short paths).
        if let (Ok(canon_path), Ok(canon_root)) = (normalized.canonicalize(), root.canonicalize()) {
            if canon_path.starts_with(&canon_root) {
                return Ok(canon_path);
            }
        }
        // Parent may exist for not-yet-created backup dirs.
        if let Some(parent) = normalized.parent() {
            if let (Ok(canon_parent), Ok(canon_root)) = (parent.canonicalize(), root.canonicalize())
            {
                if canon_parent.starts_with(&canon_root) {
                    return Ok(normalized);
                }
            }
        }
    }

    Err(
        "Path is outside the allowed directories (wallet data, home, Documents, Desktop, Downloads)."
            .to_string(),
    )
}

/// Compact LWD DB must stay under the active wallet data directory (F-11).
pub fn resolve_wallet_scoped_db_path(db_path: Option<&str>) -> Result<PathBuf, String> {
    let default = get_wallet_data_dir().join("lwd_compact.sqlite");
    let Some(raw) = db_path.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(default);
    };
    let resolved = resolve_allowlisted_user_path(raw)?;
    let wallet_root = normalize_path_components(&get_wallet_data_dir());
    if !path_is_under_root(&resolved, &wallet_root) {
        if let (Ok(canon), Ok(root)) = (
            resolved.canonicalize(),
            get_wallet_data_dir().canonicalize(),
        ) {
            if canon.starts_with(&root) {
                return Ok(canon);
            }
        }
        return Err(
            "Custom LWD database path must be under the active wallet data directory.".to_string(),
        );
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_accepts_wallet_data_subdir() {
        let dir = std::env::temp_dir().join(format!("nozy-path-sandbox-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        with_wallet_data_dir(&dir, || {
            let target = dir.join("backups");
            let resolved = resolve_allowlisted_user_path(target.to_str().unwrap()).unwrap();
            assert!(resolved.ends_with("backups"));
            let db =
                resolve_wallet_scoped_db_path(Some(target.join("lwd.sqlite").to_str().unwrap()))
                    .unwrap();
            assert!(db.ends_with("lwd.sqlite"));
        });
        let _ = std::fs::remove_dir_all(&dir);
    }
}

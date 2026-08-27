//! Default Valar hash-pinned static config sources.
//!
//! Pins must be refreshed whenever Valar rotates `static-voting-config.json`.
//! Compute with: `Get-FileHash -Algorithm SHA256` (lowercase hex) or `sha256sum`.
//! Tracking: issue #273.

use crate::config::Environment;

/// Staging static config pin (bytes verified 2026-08-14).
pub const STAGE_STATIC_PINNED: &str = concat!(
    "https://voting.valargroup.org/stage/static-voting-config.json",
    "?checksum=sha256:80890a6de9acc7293c3e2fabf870bb3e5755dbe0e69de4a59feb8f696134d4dc"
);

/// Production static config pin (bytes verified 2026-08-14).
pub const PROD_STATIC_PINNED: &str = concat!(
    "https://voting.valargroup.org/prod/static-voting-config.json",
    "?checksum=sha256:c06f1dfa2f0a30b3614aefcf00ac7e31d61ebc3cf551b3031d1b194232d1056d"
);

pub fn static_source(env: Environment, override_source: Option<&str>) -> String {
    if let Some(s) = override_source {
        return s.to_string();
    }
    match env {
        Environment::Stage => STAGE_STATIC_PINNED.to_string(),
        Environment::Prod => PROD_STATIC_PINNED.to_string(),
    }
}

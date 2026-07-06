//! Shielded pool identifiers for Orchard (legacy) and Ironwood (NU6.3+).

use serde::{Deserialize, Serialize};

/// Which Zcash shielded value pool a note or balance belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShieldedPool {
    /// Pre-Ironwood Orchard pool (V2 notes after NU6.2).
    #[default]
    Orchard,
    /// Post-NU6.3 Ironwood pool (V3 notes).
    Ironwood,
}

impl ShieldedPool {
    pub fn as_str(self) -> &'static str {
        match self {
            ShieldedPool::Orchard => "orchard",
            ShieldedPool::Ironwood => "ironwood",
        }
    }

    pub fn from_pool_id(id: &str) -> Option<Self> {
        match id {
            "orchard" => Some(ShieldedPool::Orchard),
            "ironwood" => Some(ShieldedPool::Ironwood),
            _ => None,
        }
    }

    /// Zebra `getblockchaininfo` → `valuePools[].id`.
    pub fn value_pool_id(self) -> &'static str {
        self.as_str()
    }

    /// Zebra `z_getsubtreesbyindex` pool parameter.
    pub fn subtree_pool_id(self) -> &'static str {
        self.as_str()
    }
}

impl std::fmt::Display for ShieldedPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

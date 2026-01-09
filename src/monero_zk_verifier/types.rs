// Types for ZK block verification

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Verification level for Monero blocks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationLevel {
    /// Trust RPC node (fast, for wallet operations)
    TrustRpc,

    /// Verify single block with ZK proof (for swaps)
    VerifyBlock {
        block_hash: String,
        block_height: Option<u64>,
    },

    /// Verify block chain (for high-value swaps)
    VerifyChain { start_height: u64, end_height: u64 },
}

impl VerificationLevel {
    /// Get description of verification level
    pub fn description(&self) -> &'static str {
        match self {
            VerificationLevel::TrustRpc => "Trust RPC node (fast, no verification)",
            VerificationLevel::VerifyBlock { .. } => {
                "Verify single block with ZK proof (1-2 hours)"
            }
            VerificationLevel::VerifyChain { .. } => {
                "Verify block chain with ZK proofs (multiple hours)"
            }
        }
    }

    /// Check if this level requires ZK verification
    pub fn requires_zk_proof(&self) -> bool {
        !matches!(self, VerificationLevel::TrustRpc)
    }
}

/// Result of block verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub verified: bool,

    /// Verification level used
    pub level: VerificationLevel,

    /// Time taken for verification (seconds)
    pub verification_time: Option<u64>,

    /// Proof file path (if proof was generated)
    pub proof_path: Option<PathBuf>,

    /// Error message (if verification failed)
    pub error: Option<String>,
}

impl VerificationResult {
    pub fn success(
        level: VerificationLevel,
        verification_time: u64,
        proof_path: Option<PathBuf>,
    ) -> Self {
        Self {
            verified: true,
            level,
            verification_time: Some(verification_time),
            proof_path,
            error: None,
        }
    }

    pub fn failure(level: VerificationLevel, error: String) -> Self {
        Self {
            verified: false,
            level,
            verification_time: None,
            proof_path: None,
            error: Some(error),
        }
    }
}

/// Errors that can occur during verification
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("RISC Zero prover not available: {0}")]
    ProverUnavailable(String),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Block verification failed - invalid proof-of-work")]
    InvalidProofOfWork,

    #[error("Cache proof expired - need to regenerate (valid for ~2048 blocks)")]
    CacheProofExpired,

    #[error("GPU acceleration not available: {0}")]
    GpuUnavailable(String),

    #[error("Block data not found: {0}")]
    BlockNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for ZK verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVerificationConfig {
    /// Enable ZK verification for swaps
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default verification level for swaps
    #[serde(default = "default_verification_level")]
    pub default_level: VerificationLevel,

    /// Path to RISC Zero prover binary
    pub risc_zero_prover_path: Option<String>,

    /// Use GPU acceleration (CUDA/Metal)
    #[serde(default = "default_true")]
    pub use_gpu: bool,

    /// Proof cache directory
    pub proof_cache_dir: Option<PathBuf>,

    /// Auto-generate proofs in background
    #[serde(default = "default_false")]
    pub auto_generate_proofs: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_verification_level() -> VerificationLevel {
    VerificationLevel::TrustRpc
}

impl Default for ZkVerificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_level: VerificationLevel::TrustRpc,
            risc_zero_prover_path: None,
            use_gpu: true,
            proof_cache_dir: None,
            auto_generate_proofs: false,
        }
    }
}

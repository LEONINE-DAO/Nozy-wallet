// Proof cache management for Phase 1 (cache initialization) proofs
// Phase 1 proofs are reusable for ~2048 blocks (~3 days)

use crate::error::{NozyError, NozyResult};
use crate::monero_zk_verifier::types::VerificationError;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache proof metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheProofMetadata {
    /// RandomX key hash (identifies the cache)
    pub randomx_key_hash: String,
    
    /// Block height when proof was generated
    pub block_height: u64,
    
    /// Timestamp when proof was generated
    pub generated_at: u64,
    
    /// Proof file path
    pub proof_path: PathBuf,
    
    /// Valid for ~2048 blocks from block_height
    pub valid_until_height: u64,
}

impl CacheProofMetadata {
    /// Check if cache proof is still valid for given block height
    pub fn is_valid_for(&self, block_height: u64) -> bool {
        block_height >= self.block_height && block_height < self.valid_until_height
    }
    
    /// Check if cache proof is expired
    pub fn is_expired(&self, current_height: u64) -> bool {
        current_height >= self.valid_until_height
    }
}

/// Manages cache proofs (Phase 1 proofs)
pub struct ProofCache {
    cache_dir: PathBuf,
}

impl ProofCache {
    /// Create new proof cache manager
    pub fn new(cache_dir: PathBuf) -> NozyResult<Self> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| NozyError::InvalidOperation(
                    format!("Failed to create proof cache directory: {}", e)
                ))?;
        }
        
        Ok(Self { cache_dir })
    }
    
    /// Get cache proof for given RandomX key and block height
    pub fn get_cache_proof(
        &self,
        randomx_key_hash: &str,
        block_height: u64,
    ) -> NozyResult<Option<CacheProofMetadata>> {
        let metadata_path = self.cache_dir.join(format!("cache_{}.json", randomx_key_hash));
        
        if !metadata_path.exists() {
            return Ok(None);
        }
        
        let metadata_str = fs::read_to_string(&metadata_path)
            .map_err(|e| NozyError::InvalidOperation(
                format!("Failed to read cache proof metadata: {}", e)
            ))?;
        
        let metadata: CacheProofMetadata = serde_json::from_str(&metadata_str)
            .map_err(|e| NozyError::InvalidOperation(
                format!("Failed to parse cache proof metadata: {}", e)
            ))?;
        
        // Check if proof is still valid
        if metadata.is_valid_for(block_height) {
            // Verify proof file exists
            if metadata.proof_path.exists() {
                Ok(Some(metadata))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Save cache proof metadata
    pub fn save_cache_proof(
        &self,
        randomx_key_hash: &str,
        block_height: u64,
        proof_path: PathBuf,
    ) -> NozyResult<CacheProofMetadata> {
        let valid_until_height = block_height + 2048; // Valid for ~2048 blocks
        
        let metadata = CacheProofMetadata {
            randomx_key_hash: randomx_key_hash.to_string(),
            block_height,
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            proof_path: proof_path.clone(),
            valid_until_height,
        };
        
        let metadata_path = self.cache_dir.join(format!("cache_{}.json", randomx_key_hash));
        let metadata_str = serde_json::to_string_pretty(&metadata)
            .map_err(|e| NozyError::InvalidOperation(
                format!("Failed to serialize cache proof metadata: {}", e)
            ))?;
        
        fs::write(&metadata_path, metadata_str)
            .map_err(|e| NozyError::InvalidOperation(
                format!("Failed to save cache proof metadata: {}", e)
            ))?;
        
        Ok(metadata)
    }
    
    /// Clean up expired cache proofs
    pub fn cleanup_expired(&self, current_height: u64) -> NozyResult<usize> {
        let mut cleaned = 0;
        
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| NozyError::InvalidOperation(
                format!("Failed to read cache directory: {}", e)
            ))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| NozyError::InvalidOperation(
                format!("Failed to read cache entry: {}", e)
            ))?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata_str) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<CacheProofMetadata>(&metadata_str) {
                        if metadata.is_expired(current_height) {
                            // Remove metadata and proof file
                            let _ = fs::remove_file(&path);
                            if metadata.proof_path.exists() {
                                let _ = fs::remove_file(&metadata.proof_path);
                            }
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        
        Ok(cleaned)
    }
}


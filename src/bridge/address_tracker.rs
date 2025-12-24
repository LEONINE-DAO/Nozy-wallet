// Address Tracker
// Prevents address reuse for privacy

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct AddressTracker {
    used_monero_addresses: HashSet<String>,
    used_zcash_addresses: HashSet<String>,
    storage_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddressStorage {
    monero_addresses: Vec<String>,
    zcash_addresses: Vec<String>,
}

impl AddressTracker {
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| NozyError::Storage(format!("Failed to create directory: {}", e)))?;
        }
        
        let storage_path = data_dir.join("used_addresses.json");
        
        let (used_monero, used_zcash) = if storage_path.exists() {
            Self::load_addresses(&storage_path)?
        } else {
            (HashSet::new(), HashSet::new())
        };
        
        Ok(Self {
            used_monero_addresses: used_monero,
            used_zcash_addresses: used_zcash,
            storage_path,
        })
    }
    
    /// Check if address has been used (privacy violation)
    pub fn is_address_used(&self, address: &str, is_monero: bool) -> bool {
        if is_monero {
            self.used_monero_addresses.contains(address)
        } else {
            self.used_zcash_addresses.contains(address)
        }
    }
    
    /// Mark address as used
    pub fn mark_address_used(&mut self, address: &str, is_monero: bool) -> NozyResult<()> {
        if is_monero {
            self.used_monero_addresses.insert(address.to_string());
        } else {
            self.used_zcash_addresses.insert(address.to_string());
        }
        
        self.save_addresses()?;
        Ok(())
    }
    
    /// Validate address is not reused (for swap)
    pub fn validate_address_not_reused(&self, address: &str, is_monero: bool) -> NozyResult<()> {
        if self.is_address_used(address, is_monero) {
            return Err(NozyError::InvalidOperation(format!(
                "Address reuse detected: {}\n\
                Privacy violation: Each swap must use a new address.\n\
                Please generate a new address for this swap.",
                address
            )));
        }
        
        Ok(())
    }
    
    /// Get count of used addresses
    pub fn get_stats(&self) -> (usize, usize) {
        (
            self.used_monero_addresses.len(),
            self.used_zcash_addresses.len(),
        )
    }
    
    fn load_addresses(path: &PathBuf) -> NozyResult<(HashSet<String>, HashSet<String>)> {
        let content = fs::read_to_string(path)
            .map_err(|e| NozyError::Storage(format!("Failed to read addresses: {}", e)))?;
        
        let storage: AddressStorage = serde_json::from_str(&content)
            .map_err(|e| NozyError::Storage(format!("Failed to parse addresses: {}", e)))?;
        
        Ok((
            storage.monero_addresses.into_iter().collect(),
            storage.zcash_addresses.into_iter().collect(),
        ))
    }
    
    fn save_addresses(&self) -> NozyResult<()> {
        let storage = AddressStorage {
            monero_addresses: self.used_monero_addresses.iter().cloned().collect(),
            zcash_addresses: self.used_zcash_addresses.iter().cloned().collect(),
        };
        
        let content = serde_json::to_string_pretty(&storage)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize addresses: {}", e)))?;
        
        fs::write(&self.storage_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to save addresses: {}", e)))?;
        
        Ok(())
    }
}

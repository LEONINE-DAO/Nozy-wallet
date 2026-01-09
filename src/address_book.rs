use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub usage_count: u32,
    pub notes: Option<String>,
}

impl AddressEntry {
    pub fn new(name: String, address: String, notes: Option<String>) -> Self {
        Self {
            name,
            address,
            created_at: chrono::Utc::now(),
            last_used: None,
            usage_count: 0,
            notes,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used = Some(chrono::Utc::now());
        self.usage_count += 1;
    }
}

pub struct AddressBook {
    storage_path: PathBuf,
    addresses: Arc<Mutex<HashMap<String, AddressEntry>>>,
}

impl AddressBook {
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        let book = Self {
            storage_path: data_dir.clone(),
            addresses: Arc::new(Mutex::new(HashMap::new())),
        };

        book.ensure_storage_directory()?;
        book.load_addresses()?;

        Ok(book)
    }

    fn ensure_storage_directory(&self) -> NozyResult<()> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path).map_err(|e| {
                NozyError::Storage(format!("Failed to create storage directory: {}", e))
            })?;
        }
        Ok(())
    }

    fn get_addresses_path(&self) -> PathBuf {
        self.storage_path.join("address_book.json")
    }

    fn load_addresses(&self) -> NozyResult<()> {
        let addresses_path = self.get_addresses_path();

        if addresses_path.exists() {
            let content = fs::read_to_string(&addresses_path)
                .map_err(|e| NozyError::Storage(format!("Failed to read address book: {}", e)))?;

            let stored_addresses: HashMap<String, AddressEntry> = serde_json::from_str(&content)
                .map_err(|e| NozyError::Storage(format!("Failed to parse address book: {}", e)))?;

            *self
                .addresses
                .lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))? =
                stored_addresses;
        }

        Ok(())
    }

    fn save_addresses(&self) -> NozyResult<()> {
        let addresses_path = self.get_addresses_path();
        let addresses = self
            .addresses
            .lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;

        let content = serde_json::to_string_pretty(&*addresses)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize address book: {}", e)))?;

        fs::write(&addresses_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to write address book: {}", e)))?;

        Ok(())
    }

    pub fn add_address(
        &self,
        name: String,
        address: String,
        notes: Option<String>,
    ) -> NozyResult<()> {
        if !address.starts_with("u1") && !address.starts_with("zs1") {
            return Err(NozyError::AddressParsing(
                "Invalid address format. Must be a shielded address (u1 or zs1)".to_string(),
            ));
        }

        let entry = AddressEntry::new(name.clone(), address.clone(), notes);

        {
            let mut addresses = self
                .addresses
                .lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
            addresses.insert(name, entry);
        }

        self.save_addresses()?;
        Ok(())
    }

    pub fn remove_address(&self, name: &str) -> NozyResult<bool> {
        let removed = {
            let mut addresses = self
                .addresses
                .lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
            addresses.remove(name).is_some()
        };

        if removed {
            self.save_addresses()?;
        }

        Ok(removed)
    }

    pub fn get_address(&self, name: &str) -> Option<AddressEntry> {
        self.addresses
            .lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_address: {}", e);
            })
            .ok()
            .and_then(|addresses| addresses.get(name).cloned())
    }

    pub fn get_address_by_name(&self, name: &str) -> Option<String> {
        self.get_address(name).map(|e| e.address)
    }

    pub fn list_addresses(&self) -> Vec<AddressEntry> {
        self.addresses
            .lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in list_addresses: {}", e);
            })
            .ok()
            .map(|addresses| addresses.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn search_addresses(&self, query: &str) -> Vec<AddressEntry> {
        let addresses = self
            .addresses
            .lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in search_addresses: {}", e);
            })
            .ok();

        if let Some(addresses) = addresses {
            let query_lower = query.to_lowercase();
            addresses
                .values()
                .filter(|entry| {
                    entry.name.to_lowercase().contains(&query_lower)
                        || entry.address.to_lowercase().contains(&query_lower)
                        || entry
                            .notes
                            .as_ref()
                            .map(|n| n.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn update_address_usage(&self, name: &str) -> NozyResult<bool> {
        let updated = {
            let mut addresses = self
                .addresses
                .lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
            if let Some(entry) = addresses.get_mut(name) {
                entry.mark_used();
                true
            } else {
                false
            }
        };

        if updated {
            self.save_addresses()?;
        }

        Ok(updated)
    }

    pub fn update_address(
        &self,
        name: &str,
        new_address: Option<String>,
        new_notes: Option<String>,
    ) -> NozyResult<bool> {
        let updated = {
            let mut addresses = self
                .addresses
                .lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
            if let Some(entry) = addresses.get_mut(name) {
                if let Some(addr) = new_address {
                    if !addr.starts_with("u1") && !addr.starts_with("zs1") {
                        return Err(NozyError::AddressParsing(
                            "Invalid address format. Must be a shielded address (u1 or zs1)"
                                .to_string(),
                        ));
                    }
                    entry.address = addr;
                }
                if let Some(notes) = new_notes {
                    entry.notes = Some(notes);
                }
                true
            } else {
                false
            }
        };

        if updated {
            self.save_addresses()?;
        }

        Ok(updated)
    }

    pub fn count(&self) -> usize {
        self.addresses
            .lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in count: {}", e);
            })
            .ok()
            .map(|addresses| addresses.len())
            .unwrap_or(0)
    }
}

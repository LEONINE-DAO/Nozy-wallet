// Simple caching utilities for API calls

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

pub struct SimpleCache<T> {
    entries: Arc<Mutex<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

impl<T: Clone> SimpleCache<T> {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.lock().ok()?;
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            } else {
                entries.remove(key);
            }
        }
        None
    }
    
    pub fn set(&self, key: String, value: T) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, CacheEntry {
                value,
                expires_at: Instant::now() + self.default_ttl,
            });
        }
    }
    
    pub fn set_with_ttl(&self, key: String, value: T, ttl: Duration) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            });
        }
    }
    
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }
    
    pub fn remove_expired(&self) -> usize {
        let mut entries = match self.entries.lock() {
            Ok(e) => e,
            Err(_) => return 0,
        };
        
        let now = Instant::now();
        let keys_to_remove: Vec<String> = entries.iter()
            .filter(|(_, entry)| entry.expires_at <= now)
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in &keys_to_remove {
            entries.remove(key);
        }
        
        keys_to_remove.len()
    }
}

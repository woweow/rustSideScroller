use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ValueWithTTL {
    value: String,
    expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvStore {
    #[serde(skip)]
    file_path: String,
    store: HashMap<String, ValueWithTTL>,
}

impl KvStore {
    pub fn new(file_path: &str) -> Self {
        let store = if Path::new(file_path).exists() {
            let contents = fs::read_to_string(file_path)
                .unwrap_or_else(|_| String::from("{}"));
            serde_json::from_str(&contents).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        let mut kv_store = KvStore {
            store,
            file_path: file_path.to_string(),
        };
        
        // Clean expired entries on load
        kv_store.cleanup_expired();
        kv_store
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).and_then(|value_with_ttl| {
            if self.is_expired(value_with_ttl) {
                None
            } else {
                Some(value_with_ttl.value.clone())
            }
        })
    }

    pub fn set(&mut self, key: String, value: String) {
        self.set_with_ttl(key, value, None);
    }

    pub fn set_with_ttl(&mut self, key: String, value: String, ttl_seconds: Option<u64>) {
        let expires_at = ttl_seconds.map(|ttl| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + ttl
        });

        self.store.insert(key, ValueWithTTL { value, expires_at });
        self.save();
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        let result = self.store.remove(key).map(|v| v.value);
        self.save();
        result
    }

    fn save(&self) {
        if let Ok(serialized) = serde_json::to_string(&self.store) {
            fs::write(&self.file_path, serialized).ok();
        }
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.store
            .iter()
            .filter(|(_, value_with_ttl)| !self.is_expired(value_with_ttl))
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    fn is_expired(&self, value: &ValueWithTTL) -> bool {
        if let Some(expires_at) = value.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            expires_at <= now
        } else {
            false
        }
    }

    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<String> = self.store
            .iter()
            .filter(|(_, value)| self.is_expired(value))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.store.remove(&key);
        }
        
        self.save();
    }

    // Helper method to get TTL information
    pub fn get_ttl(&self, key: &str) -> Option<u64> {
        self.store.get(key).and_then(|value_with_ttl| {
            if self.is_expired(value_with_ttl) {
                None
            } else {
                value_with_ttl.expires_at.map(|expires_at| {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    expires_at.saturating_sub(now)
                })
            }
        })
    }
} 
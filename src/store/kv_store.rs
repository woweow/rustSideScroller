use std::collections::HashMap;
use std::time::SystemTime;
use crate::persistence::file::FileStorage;
use super::types::Value;

pub struct KvStore {
    store: HashMap<String, Value>,
    storage: FileStorage,
}

impl KvStore {
    pub fn new() -> Self {
        let storage = FileStorage::new();
        let store = storage.load().unwrap_or_default();
        let mut kv_store = KvStore { store, storage };
        kv_store.cleanup_expired();
        kv_store
    }

    pub fn set(&mut self, key: String, value: String, ttl: Option<u64>) -> Result<(), String> {
        let json_value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
        
        let expires_at = ttl.map(|seconds| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() + seconds
        });

        self.store.insert(key, Value {
            data: json_value,
            expires_at,
        });
        self.storage.save(&self.store).expect("Failed to save to disk");
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.cleanup_expired();
        if let Some(value) = self.store.get(key) {
            if self.is_expired(value) {
                self.delete(key);
                None
            } else {
                Some(serde_json::to_string(&value.data).unwrap())
            }
        } else {
            None
        }
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        let result = self.store.remove(key)
            .map(|v| serde_json::to_string(&v.data).unwrap());
        self.storage.save(&self.store).expect("Failed to save to disk");
        result
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn get_all(&self) -> &HashMap<String, Value> {
        &self.store
    }

    fn is_expired(&self, value: &Value) -> bool {
        value.expires_at.map_or(false, |expires_at| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() > expires_at
        })
    }

    pub fn cleanup_expired(&mut self) {
        let expired_keys: Vec<String> = self.store
            .iter()
            .filter(|(_, value)| self.is_expired(value))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.delete(&key);
        }
    }
} 
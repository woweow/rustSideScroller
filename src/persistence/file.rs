use std::collections::HashMap;
use std::fs;
use crate::store::types::Value;

const STORAGE_PATH: &str = "kv_store.json";

pub struct FileStorage;

impl FileStorage {
    pub fn new() -> Self {
        FileStorage
    }

    pub fn save(&self, store: &HashMap<String, Value>) -> std::io::Result<()> {
        let contents = serde_json::to_string(store)?;
        fs::write(STORAGE_PATH, contents)
    }

    pub fn load(&self) -> Option<HashMap<String, Value>> {
        match fs::read_to_string(STORAGE_PATH) {
            Ok(contents) => serde_json::from_str(&contents).ok(),
            Err(_) => None,
        }
    }
} 
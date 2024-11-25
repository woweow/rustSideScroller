use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Clone)]
pub struct Value {
    pub data: JsonValue,
    pub expires_at: Option<u64>, // Unix timestamp in seconds
} 
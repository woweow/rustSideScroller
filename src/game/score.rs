use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::time::SystemTime;
use crate::store::kv_store::KvStore;
use rand::{thread_rng, Rng};

const HISCORE_PREFIX: &str = "hiscore:";
const HISCORE_TTL_KEY: &str = "hiscore_ttl";
const DEFAULT_TTL: u64 = 300; // 5 minutes in seconds
const NANOID_LENGTH: usize = 8;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HiScore {
    pub name: String,
    pub score: u32,
}

pub struct ScoreManager {
    store: Arc<Mutex<KvStore>>,
}

impl ScoreManager {
    pub fn new(store: Arc<Mutex<KvStore>>) -> Self {
        Self { store }
    }

    fn generate_nanoid() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut rng = thread_rng();
        (0..NANOID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn sanitize_name(input: &str) -> String {
        // Keep only a-z and A-Z characters
        let sanitized: String = input
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect();

        if sanitized.is_empty() {
            // If no valid characters, generate a random ID
            let nanoid = Self::generate_nanoid();
            println!("No valid characters in name. Using generated ID: {}", nanoid);
            nanoid
        } else {
            sanitized
        }
    }

    fn get_valid_name() -> String {
        loop {
            print!("Enter your name (letters only): ");
            io::stdout().flush().unwrap();

            let mut name = String::new();
            io::stdin().read_line(&mut name).unwrap();
            let name = name.trim();

            let sanitized = Self::sanitize_name(name);
            if !sanitized.is_empty() {
                if sanitized != name {
                    println!("Name sanitized to: {}", sanitized);
                }
                return sanitized;
            }
        }
    }

    fn get_ttl(&self) -> u64 {
        let mut store = self.store.lock().unwrap();
        store.get(HISCORE_TTL_KEY)
            .and_then(|ttl_str| ttl_str.parse().ok())
            .unwrap_or(DEFAULT_TTL)
    }

    fn get_hiscores(&self) -> Vec<(HiScore, Option<u64>)> {
        let store = self.store.lock().unwrap();
        let mut scores = Vec::new();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (key, value) in store.get_all() {
            if key.starts_with(HISCORE_PREFIX) {
                let data_str = value.data.to_string();
                let data_str = data_str.trim_matches('"');
                
                if let Ok(score) = serde_json::from_str::<HiScore>(data_str) {
                    let remaining_ttl = value.expires_at.map(|expires_at| {
                        if expires_at > now {
                            expires_at - now
                        } else {
                            0
                        }
                    });
                    scores.push((score, remaining_ttl));
                }
            }
        }

        scores.sort_by(|a, b| b.0.score.cmp(&a.0.score));
        scores
    }

    fn save_hiscore(&self, score: HiScore) {
        let ttl = self.get_ttl();
        let mut store = self.store.lock().unwrap();
        
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let key = format!("{}{}-{}-{}", HISCORE_PREFIX, score.name, score.score, timestamp);
        let json = serde_json::to_string(&score).unwrap();
        
        store.set(key, json, Some(ttl))
            .expect("Failed to save high score");
    }

    pub fn handle_new_score(&self, score: u32) -> Vec<(String, u32, Option<u64>)> {
        let hiscores = self.get_hiscores();
        let is_high_score = hiscores.len() < 3 || 
                           hiscores.is_empty() || 
                           score > hiscores.last().map_or(0, |last| last.0.score);

        if is_high_score {
            println!("\nCongratulations! You made the top 3!");
            println!(); // Add a blank line before name prompt
            let name = Self::get_valid_name();
            self.save_hiscore(HiScore { name, score });
        }

        // Return the high scores for the UI to display
        self.get_hiscores()
            .iter()
            .take(3)
            .map(|(score, ttl)| (score.name.clone(), score.score, *ttl))
            .collect()
    }

    pub fn set_ttl(&self, ttl: u64) {
        let mut store = self.store.lock().unwrap();
        store.set(
            HISCORE_TTL_KEY.to_string(),
            ttl.to_string(),
            None,
        ).expect("Failed to save high score TTL");
    }
} 
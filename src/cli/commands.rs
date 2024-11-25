use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use crate::store::kv_store::KvStore;

#[allow(dead_code)]
pub struct CLI {
    store: Arc<Mutex<KvStore>>,
}

pub fn split_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in input.chars() {
        match c {
            '\\' if !escaped => escaped = true,
            '\'' | '"' if !escaped => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
            }
            _ => {
                if escaped && c != '\\' {
                    current.push('\\');
                }
                current.push(c);
                escaped = false;
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

#[allow(dead_code)]
impl CLI {
    pub fn new(store: Arc<Mutex<KvStore>>) -> Self {
        CLI { store }
    }

    pub fn run(&self) {
        self.print_help();

        loop {
            if !self.process_command() {
                break;
            }
        }
    }

    fn print_help(&self) {
        println!("Simple Key-Value Store");
        println!("Available commands:");
        println!("  set <key> <value> [ttl_seconds]");
        println!("  get <key>");
        println!("  delete <key>");
        println!("  list");
        println!("  set-hiscore-ttl <seconds>");
        println!("  exit");
    }

    fn process_command(&self) -> bool {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        let parts = split_command(input);

        if parts.is_empty() {
            return true;
        }

        match parts[0].as_str() {
            "set" => {
                let mut store = self.store.lock().unwrap();
                self.handle_set(&parts, &mut store)
            },
            "get" => {
                let mut store = self.store.lock().unwrap();
                self.handle_get(&parts, &mut store)
            },
            "delete" => {
                let mut store = self.store.lock().unwrap();
                self.handle_delete(&parts, &mut store)
            },
            "list" => {
                let store = self.store.lock().unwrap();
                self.handle_list(&store)
            },
            "set-hiscore-ttl" => self.handle_set_hiscore_ttl(&parts),
            "exit" => return false,
            _ => self.print_help(),
        }

        true
    }

    fn handle_set(&self, parts: &[String], store: &mut KvStore) {
        if parts.len() < 3 {
            println!("Usage: set <key> <value> [ttl_seconds]");
            println!("Note: value must be a valid JSON string");
            return;
        }

        let key = parts[1].clone();
        
        // Find if the last part is a number (TTL)
        let (value, ttl) = if parts.len() > 3 {
            if let Ok(ttl) = parts.last().unwrap().parse::<u64>() {
                // If last part is a number, exclude it from value
                let value = parts[2..parts.len()-1].join(" ");
                // Remove surrounding quotes if present
                let value = value.trim_matches(|c| c == '\'' || c == '"');
                (value.to_string(), Some(ttl))
            } else {
                let value = parts[2..].join(" ");
                let value = value.trim_matches(|c| c == '\'' || c == '"');
                (value.to_string(), None)
            }
        } else {
            let value = parts[2].trim_matches(|c| c == '\'' || c == '"');
            (value.to_string(), None)
        };
        
        match store.set(key.clone(), value.clone(), ttl) {
            Ok(_) => println!("Set \"{}\" = {}{}", 
                key, 
                value,
                ttl.map_or(String::new(), |t| format!(" (expires in {} seconds)", t))
            ),
            Err(e) => println!("Error: Invalid JSON value: {}", e),
        }
    }

    fn handle_get(&self, parts: &[String], store: &mut KvStore) {
        if parts.len() != 2 {
            println!("Usage: get <key>");
            return;
        }
        match store.get(&parts[1]) {
            Some(value) => println!("\"{}\" = \"{}\"", parts[1], value),
            None => println!("Key \"{}\" not found", parts[1]),
        }
    }

    fn handle_delete(&self, parts: &[String], store: &mut KvStore) {
        if parts.len() != 2 {
            println!("Usage: delete <key>");
            return;
        }
        let key = &parts[1];
        match store.delete(key) {
            Some(value) => println!("Deleted \"{}\" = \"{}\"", key, value),
            None => println!("Key \"{}\" not found", key),
        }
    }

    fn handle_list(&self, store: &KvStore) {
        if store.is_empty() {
            println!("Store is empty");
            return;
        }

        println!("Store contents:");
        for (key, value) in store.get_all() {
            let ttl_info = value.expires_at.map_or(String::new(), |expires_at| {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if expires_at > now {
                    format!(" (expires in {} seconds)", expires_at - now)
                } else {
                    " (expired)".to_string()
                }
            });
            println!("  \"{}\" = \"{}\"{}",
                key,
                value.data,
                ttl_info
            );
        }
        println!("Total items: {}", store.len());
    }

    fn handle_set_hiscore_ttl(&self, parts: &[String]) {
        if parts.len() != 2 {
            println!("Usage: set-hiscore-ttl <seconds>");
            return;
        }

        if let Ok(ttl) = parts[1].parse::<u64>() {
            // Drop the store lock before creating ScoreManager
            {
                let mut store = self.store.lock().unwrap();
                store.set(
                    "hiscore_ttl".to_string(),
                    ttl.to_string(),
                    None,
                ).expect("Failed to save high score TTL");
            }
            println!("High score TTL set to {} seconds", ttl);
        } else {
            println!("Invalid TTL value. Please provide a positive number.");
        }
    }
} 
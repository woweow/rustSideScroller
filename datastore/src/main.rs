use clap::{Parser, Subcommand};
use simple_kv_store::KvStore;
use std::io::{self, Write};
use std::process;

#[derive(Parser)]
#[command(name = "kv")]
#[command(about = "A simple key-value store CLI", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "store.json")]
    file: String,
}

#[derive(Subcommand)]
enum Command {
    /// Get a value by key
    Get {
        /// The key to look up
        key: String,
    },
    /// Set a key-value pair
    Set {
        /// The key to set
        key: String,
        /// The value to set
        value: String,
        /// Optional TTL in seconds
        #[arg(short, long)]
        ttl: Option<u64>,
    },
    /// Delete a key-value pair
    Delete {
        /// The key to delete
        key: String,
    },
    /// List all key-value pairs
    List,
    /// Get TTL for a key
    Ttl {
        /// The key to check TTL for
        key: String,
    },
    /// Exit the shell
    Exit,
    /// Show help message
    Help,
}

fn print_help() {
    println!("Available commands:");
    println!("  get <key>                     Get a value by key");
    println!("  set <key> <value> [--ttl <seconds>]  Set a key-value pair with optional TTL");
    println!("  delete <key>                  Delete a key-value pair");
    println!("  list                          List all key-value pairs");
    println!("  ttl <key>                     Get TTL for a key");
    println!("  exit                          Exit the shell");
    println!("  help                          Show this help message");
}

fn parse_input(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "get" if parts.len() == 2 => Some(Command::Get {
            key: parts[1].to_string(),
        }),
        "set" => {
            if parts.len() >= 3 {
                let key = parts[1].to_string();
                let value = parts[2].to_string();
                let mut ttl = None;

                if parts.len() >= 5 && parts[3] == "--ttl" {
                    if let Ok(ttl_val) = parts[4].parse() {
                        ttl = Some(ttl_val);
                    }
                }

                Some(Command::Set { key, value, ttl })
            } else {
                println!("Usage: set <key> <value> [--ttl <seconds>]");
                None
            }
        }
        "delete" if parts.len() == 2 => Some(Command::Delete {
            key: parts[1].to_string(),
        }),
        "list" if parts.len() == 1 => Some(Command::List),
        "ttl" if parts.len() == 2 => Some(Command::Ttl {
            key: parts[1].to_string(),
        }),
        "exit" | "quit" => Some(Command::Exit),
        "help" => Some(Command::Help),
        _ => {
            println!("Unknown command. Type 'help' for available commands.");
            None
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut store = KvStore::new(&cli.file);
    let mut input = String::new();

    println!("KV Store Shell (Type 'help' for commands)");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let command = match parse_input(&input) {
            Some(cmd) => cmd,
            None => continue,
        };

        match command {
            Command::Get { key } => {
                match store.get(&key) {
                    Some(value) => println!("{}", value),
                    None => println!("Key '{}' not found", key),
                }
            }
            Command::Set { key, value, ttl } => {
                store.set_with_ttl(key.clone(), value, ttl);
                println!("Set '{}' successfully", key);
            }
            Command::Delete { key } => {
                match store.delete(&key) {
                    Some(_) => println!("Deleted '{}' successfully", key),
                    None => println!("Key '{}' not found", key),
                }
            }
            Command::List => {
                let pairs = store.list();
                if pairs.is_empty() {
                    println!("Store is empty");
                } else {
                    for (key, value) in pairs {
                        match store.get_ttl(&key) {
                            Some(ttl) => println!("{}: {} (TTL: {}s)", key, value, ttl),
                            None => println!("{}: {} (No TTL)", key, value),
                        }
                    }
                }
            }
            Command::Ttl { key } => {
                match store.get_ttl(&key) {
                    Some(ttl) => println!("TTL for '{}': {} seconds", key, ttl),
                    None => println!("Key '{}' not found or has no TTL", key),
                }
            }
            Command::Exit => {
                println!("Goodbye!");
                break;
            }
            Command::Help => {
                print_help();
            }
        }
    }
} 
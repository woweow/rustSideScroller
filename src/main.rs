mod cli;
mod store;
mod persistence;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use cli::commands::CLI;
use store::kv_store::KvStore;

fn start_cleanup_thread(store: Arc<Mutex<KvStore>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(mut store) = store.lock() {
                store.cleanup_expired();
            }
        }
    });
}

fn main() {
    let store = Arc::new(Mutex::new(KvStore::new()));
    start_cleanup_thread(store.clone());
    
    let cli = CLI::new(store);
    cli.run();
} 
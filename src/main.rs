mod core;
mod ui;
mod cli;
mod server;

use std::{thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::Hide,
};
use std::sync::{Arc, Mutex};
use simple_kv_store::KvStore;
use std::env;

use crate::cli::{CLI, GameRunner};
use crate::core::Game;
use crate::ui::{render_game, handle_input, ask_play_again};
use crate::server::GameServer;

pub const GAME_WIDTH: usize = 40;
pub const FRAME_DURATION: Duration = Duration::from_millis(200);
pub const OBSTACLE_CHANCE: f64 = 0.3;
pub const INITIAL_OBSTACLE_DENSITY: f64 = 0.2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(KvStore::new()?));
    
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--cli") => run_cli_mode(store),
        Some("--server") => {
            let port = args.get(2)
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000);
            run_server_mode(store, port).await
        }
        Some("--db") => run_db_mode(store),
        _ => run_terminal_mode(store),
    }
}

fn run_cli_mode(store: Arc<Mutex<KvStore>>) -> Result<(), Box<dyn std::error::Error>> {
    let runner = GameRunner::new(store);
    runner.run();
    Ok(())
}

fn run_db_mode(store: Arc<Mutex<KvStore>>) -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::new(store);
    cli.run();
    Ok(())
}

async fn run_server_mode(store: Arc<Mutex<KvStore>>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let server = GameServer::new(store);
    server.run(port).await;
    Ok(())
}

fn run_terminal_mode(store: Arc<Mutex<KvStore>>) -> Result<(), Box<dyn std::error::Error>> {
    let runner = GameRunner::new(store);
    runner.run();
    Ok(())
} 
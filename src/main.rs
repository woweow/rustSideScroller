mod core;
mod ui;
mod cli;

use std::{thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::Hide,
};
use std::sync::{Arc, Mutex};
use simple_kv_store::KvStore;
use std::env;

use crate::cli::commands::CLI;
use crate::core::{Game, GameState};
use crate::ui::{render_game, handle_input, ask_play_again};

pub const GAME_WIDTH: usize = 40;
pub const FRAME_DURATION: Duration = Duration::from_millis(200);
pub const OBSTACLE_CHANCE: f64 = 0.3;
pub const INITIAL_OBSTACLE_DENSITY: f64 = 0.2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(KvStore::new()?));
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--cli" {
        let cli = CLI::new(store);
        cli.run();
        Ok(())
    } else {
        enable_raw_mode()?;
        execute!(std::io::stdout(), Hide)?;
        
        loop {
            let mut game = Game::new(store.clone());
            
            while !game.get_state().is_game_over {
                if let Some(movement) = handle_input(Duration::from_millis(10)) {
                    game.handle_input(movement);
                }
                
                render_game(&game.get_state());
                game.update();
                thread::sleep(FRAME_DURATION);
            }
            
            disable_raw_mode()?;
            
            let high_scores = game.handle_game_over();
            println!("\nGame Over! Final score: {}", game.get_state().score);
            
            if !high_scores.is_empty() {
                println!("\nHigh Scores:");
                for (i, (name, score, ttl)) in high_scores.iter().enumerate() {
                    let ttl_info = ttl.map_or(String::new(), |t| format!(" (expires in {}s)", t));
                    println!("{}. {} - {}{}", i + 1, name, score, ttl_info);
                }
            }
            
            println!();
            
            if !ask_play_again() {
                break;
            }
            
            enable_raw_mode()?;
            execute!(std::io::stdout(), Clear(ClearType::All))?;
        }
        
        Ok(())
    }
} 
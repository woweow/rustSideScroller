mod cli;
mod store;
mod persistence;
mod game;

use std::{io::{self, Write}, thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::{Hide, Show},
};
use std::sync::{Arc, Mutex};
use crate::store::kv_store::KvStore;
use std::env;
use crate::cli::commands::CLI;
use crate::game::Game;

pub const GAME_WIDTH: usize = 40;
pub const FRAME_DURATION: Duration = Duration::from_millis(200);
pub const OBSTACLE_CHANCE: f64 = 0.3;
pub const INITIAL_OBSTACLE_DENSITY: f64 = 0.2;

fn ask_play_again() -> bool {
    print!("Play again? (y/n): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
}

fn main() {
    let store = Arc::new(Mutex::new(KvStore::new()));
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--cli" {
        let cli = CLI::new(store);
        cli.run();
    } else {
        execute!(io::stdout(), Hide).unwrap();
        
        loop {
            let mut game = Game::new(store.clone());
            
            while !game.is_collision() {
                game.handle_input();
                game.render();
                game.update();
                thread::sleep(FRAME_DURATION);
            }
            
            game.handle_game_over();
            println!();
            
            if !ask_play_again() {
                break;
            }
            
            execute!(io::stdout(), Clear(ClearType::All)).unwrap();
        }
    }
} 
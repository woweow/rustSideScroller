mod cli;
mod game;

use std::{io::{self, Write}, thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode},
};
use std::sync::{Arc, Mutex};
use simple_kv_store::KvStore;
use std::env;
use crate::cli::commands::CLI;
use crate::game::{Game, GameState, PlayerMove};

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

fn handle_cli_input(duration: Duration) -> Option<PlayerMove> {
    if poll(duration).unwrap() {
        if let Ok(Event::Key(key_event)) = read() {
            return match key_event.code {
                KeyCode::Up => Some(PlayerMove::Up),
                KeyCode::Down => Some(PlayerMove::Down),
                KeyCode::Char('q') => Some(PlayerMove::Quit),
                _ => None,
            };
        }
    }
    None
}

fn render_game(state: &GameState) {
    execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    
    println!("Score: {}", state.score);
    
    execute!(io::stdout(), MoveTo(0, 1)).unwrap();
    
    for (i, &has_obstacle) in state.top_row.iter().enumerate() {
        if state.player_pos == (i, 0) {
            print!("x");
        } else if has_obstacle {
            print!("-");
        } else {
            print!(" ");
        }
    }
    
    execute!(io::stdout(), MoveTo(0, 2)).unwrap();
    
    for (i, &has_obstacle) in state.bottom_row.iter().enumerate() {
        if state.player_pos == (i, 1) {
            print!("x");
        } else if has_obstacle {
            print!("-");
        } else {
            print!(" ");
        }
    }
    
    io::stdout().flush().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(KvStore::new()?));
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--cli" {
        let cli = CLI::new(store);
        cli.run();
        Ok(())
    } else {
        enable_raw_mode()?;
        execute!(io::stdout(), Hide)?;
        
        loop {
            let mut game = Game::new(store.clone());
            
            while !game.get_state().is_game_over {
                if let Some(movement) = handle_cli_input(Duration::from_millis(10)) {
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
            execute!(io::stdout(), Clear(ClearType::All))?;
        }
        
        Ok(())
    }
} 
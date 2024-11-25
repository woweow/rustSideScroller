use std::{thread, time::Duration};
use std::sync::{Arc, Mutex};
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
};
use std::io::stdout;
use simple_kv_store::KvStore;
use crate::core::Game;
use crate::ui::{render_game, handle_input, ask_play_again};
use crate::{FRAME_DURATION};

pub struct GameRunner {
    store: Arc<Mutex<KvStore>>,
}

impl GameRunner {
    pub fn new(store: Arc<Mutex<KvStore>>) -> Self {
        Self { store }
    }

    pub fn run(&self) {
        loop {
            let mut game = Game::new(self.store.clone());
            
            enable_raw_mode().unwrap();
            execute!(stdout(), Hide).unwrap();
            
            while !game.get_state().is_game_over {
                if let Some(movement) = handle_input(Duration::from_millis(10)) {
                    game.handle_input(movement);
                }
                
                render_game(&game.get_state());
                game.update();
                thread::sleep(FRAME_DURATION);
            }
            
            // Restore normal terminal mode for input
            disable_raw_mode().unwrap();
            execute!(stdout(), Show).unwrap();
            
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
        }
    }
} 
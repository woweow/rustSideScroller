mod cli;
mod store;
mod persistence;

use std::{io::{self, Write}, thread, time::Duration};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
    event::{poll, read, Event, KeyCode},
};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use crate::store::kv_store::KvStore;
use std::env;

const GAME_WIDTH: usize = 40;
const GAME_HEIGHT: usize = 2;
const FRAME_DURATION: Duration = Duration::from_millis(200);
const OBSTACLE_CHANCE: f64 = 0.3;
const INITIAL_OBSTACLE_DENSITY: f64 = 0.2;
const HISCORES_KEY: &str = "hiscores";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HiScore {
    name: String,
    score: u32,
}

struct Game {
    player_pos: (usize, usize),
    top_row: Vec<bool>,
    bottom_row: Vec<bool>,
    score: u32,
    store: Arc<Mutex<KvStore>>,
}

impl Game {
    fn new(store: Arc<Mutex<KvStore>>) -> Self {
        let mut rng = rand::thread_rng();
        
        let top_row = (0..GAME_WIDTH)
            .map(|i| {
                if i <= 10 {
                    false
                } else {
                    rng.gen_bool(INITIAL_OBSTACLE_DENSITY)
                }
            })
            .collect();

        let bottom_row = (0..GAME_WIDTH)
            .map(|i| {
                if i <= 10 {
                    false
                } else {
                    rng.gen_bool(INITIAL_OBSTACLE_DENSITY)
                }
            })
            .collect();

        Game {
            player_pos: (1, 1),
            top_row,
            bottom_row,
            score: 0,
            store,
        }
    }

    fn update(&mut self) {
        self.score += 1;
        self.top_row.rotate_left(1);
        self.bottom_row.rotate_left(1);

        let mut rng = rand::thread_rng();
        self.top_row[GAME_WIDTH - 1] = rng.gen_bool(OBSTACLE_CHANCE);
        self.bottom_row[GAME_WIDTH - 1] = rng.gen_bool(OBSTACLE_CHANCE);

        if self.top_row[GAME_WIDTH - 1] && self.bottom_row[GAME_WIDTH - 1] {
            if rng.gen_bool(0.5) {
                self.top_row[GAME_WIDTH - 1] = false;
            } else {
                self.bottom_row[GAME_WIDTH - 1] = false;
            }
        }
    }

    fn handle_input(&mut self) {
        if poll(Duration::from_millis(10)).unwrap() {
            if let Ok(Event::Key(key_event)) = read() {
                match key_event.code {
                    KeyCode::Up => self.player_pos.1 = 0,
                    KeyCode::Down => self.player_pos.1 = 1,
                    _ => {}
                }
            }
        }
    }

    fn is_collision(&self) -> bool {
        let (x, y) = self.player_pos;
        (y == 0 && self.top_row[x]) || (y == 1 && self.bottom_row[x])
    }

    fn render(&self) {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        
        println!("Score: {}", self.score);
        
        for (i, &has_obstacle) in self.top_row.iter().enumerate() {
            if self.player_pos == (i, 0) {
                print!("x");
            } else if has_obstacle {
                print!("-");
            } else {
                print!(" ");
            }
        }
        println!();

        for (i, &has_obstacle) in self.bottom_row.iter().enumerate() {
            if self.player_pos == (i, 1) {
                print!("x");
            } else if has_obstacle {
                print!("-");
            } else {
                print!(" ");
            }
        }
        println!();
        
        io::stdout().flush().unwrap();
    }

    fn get_hiscores(&self) -> Vec<HiScore> {
        let mut store = self.store.lock().unwrap();
        if let Some(scores_str) = store.get(HISCORES_KEY) {
            serde_json::from_str(&scores_str).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_hiscores(&self, scores: Vec<HiScore>) {
        let mut store = self.store.lock().unwrap();
        store.set(
            HISCORES_KEY.to_string(),
            serde_json::to_string(&scores).unwrap(),
            None,
        ).expect("Failed to save high scores");
    }

    fn handle_game_over(&self) {
        execute!(io::stdout(), Show).unwrap();
        println!("\nGame Over! Final score: {}", self.score);

        let mut hiscores = self.get_hiscores();
        let is_high_score = hiscores.is_empty() || hiscores.len() < 3 || self.score > hiscores.last().unwrap().score;

        if is_high_score {
            println!("\nCongratulations! You made the top 3!");
            print!("Enter your name: ");
            io::stdout().flush().unwrap();

            let mut name = String::new();
            io::stdin().read_line(&mut name).unwrap();
            let name = name.trim().to_string();

            hiscores.push(HiScore {
                name,
                score: self.score,
            });

            // Sort by score in descending order
            hiscores.sort_by(|a, b| b.score.cmp(&a.score));
            
            // Keep only top 3
            hiscores.truncate(3);

            self.save_hiscores(hiscores.clone());
        }

        println!("\nHigh Scores:");
        for (i, score) in hiscores.iter().enumerate() {
            println!("{}. {} - {}", i + 1, score.name, score.score);
        }
    }
}

fn clear_hiscores(store: &Arc<Mutex<KvStore>>) {
    let mut store = store.lock().unwrap();
    store.delete(HISCORES_KEY);
    println!("High scores cleared.");
}

fn ask_play_again() -> bool {
    print!("Play again? (y/n): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let store = Arc::new(Mutex::new(KvStore::new()));

    if args.len() > 1 && args[1] == "--clear-hiscores" {
        clear_hiscores(&store);
        return Ok(());
    }

    loop {  // Main game loop
        execute!(io::stdout(), Hide)?;

        let mut game = Game::new(store.clone());
        let mut last_update = std::time::Instant::now();

        loop {  // Single game loop
            game.handle_input();

            if last_update.elapsed() >= FRAME_DURATION {
                game.update();
                last_update = std::time::Instant::now();

                if game.is_collision() {
                    game.handle_game_over();
                    break;
                }
            }

            game.render();
            thread::sleep(Duration::from_millis(16));
        }

        if !ask_play_again() {
            break;
        }
        
        // Clear the screen before starting new game
        execute!(io::stdout(), Clear(ClearType::All))?;
    }

    Ok(())
} 
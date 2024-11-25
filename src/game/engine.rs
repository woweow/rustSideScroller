use std::sync::{Arc, Mutex};
use std::time::Duration;
use rand::Rng;
use crossterm::event::{poll, read, Event, KeyCode};
use crate::store::kv_store::KvStore;
use super::renderer::GameRenderer;
use super::score::ScoreManager;
use crate::{GAME_WIDTH, OBSTACLE_CHANCE, INITIAL_OBSTACLE_DENSITY};

pub struct Game {
    player_pos: (usize, usize),
    top_row: Vec<bool>,
    bottom_row: Vec<bool>,
    score: u32,
    store: Arc<Mutex<KvStore>>,
    renderer: GameRenderer,
    score_manager: ScoreManager,
}

impl Game {
    pub fn new(store: Arc<Mutex<KvStore>>) -> Self {
        let mut rng = rand::thread_rng();
        
        let top_row = (0..GAME_WIDTH)
            .map(|i| {
                if i <= 10 { false } 
                else { rng.gen_bool(INITIAL_OBSTACLE_DENSITY) }
            })
            .collect();

        let bottom_row = (0..GAME_WIDTH)
            .map(|i| {
                if i <= 10 { false }
                else { rng.gen_bool(INITIAL_OBSTACLE_DENSITY) }
            })
            .collect();

        Game {
            player_pos: (1, 1),
            top_row,
            bottom_row,
            score: 0,
            store: store.clone(),
            renderer: GameRenderer::new(),
            score_manager: ScoreManager::new(store),
        }
    }

    pub fn update(&mut self) {
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

    pub fn handle_input(&mut self) {
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

    pub fn is_collision(&self) -> bool {
        let (x, y) = self.player_pos;
        (y == 0 && self.top_row[x]) || (y == 1 && self.bottom_row[x])
    }

    pub fn render(&self) {
        self.renderer.render(
            self.score,
            &self.top_row,
            &self.bottom_row,
            self.player_pos
        );
    }

    pub fn handle_game_over(&self) {
        self.renderer.show_game_over(self.score);
        self.score_manager.handle_new_score(self.score);
    }
} 
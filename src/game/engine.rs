use std::sync::{Arc, Mutex};
use rand::Rng;
use simple_kv_store::KvStore;
use super::score::ScoreManager;
use crate::{GAME_WIDTH, OBSTACLE_CHANCE, INITIAL_OBSTACLE_DENSITY};

#[derive(Clone, Copy)]
pub enum PlayerMove {
    Up,
    Down,
    Quit,
}

#[derive(Clone)]
pub struct GameState {
    pub player_pos: (usize, usize),
    pub top_row: Vec<bool>,
    pub bottom_row: Vec<bool>,
    pub score: u32,
    pub is_game_over: bool,
}

pub struct Game {
    state: GameState,
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
            state: GameState {
                player_pos: (1, 1),
                top_row,
                bottom_row,
                score: 0,
                is_game_over: false,
            },
            score_manager: ScoreManager::new(store),
        }
    }

    pub fn get_state(&self) -> GameState {
        self.state.clone()
    }

    pub fn update(&mut self) {
        if self.state.is_game_over {
            return;
        }

        self.state.score += 1;
        self.state.top_row.rotate_left(1);
        self.state.bottom_row.rotate_left(1);

        let mut rng = rand::thread_rng();
        self.state.top_row[GAME_WIDTH - 1] = rng.gen_bool(OBSTACLE_CHANCE);
        self.state.bottom_row[GAME_WIDTH - 1] = rng.gen_bool(OBSTACLE_CHANCE);

        if self.state.top_row[GAME_WIDTH - 1] && self.state.bottom_row[GAME_WIDTH - 1] {
            if rng.gen_bool(0.5) {
                self.state.top_row[GAME_WIDTH - 1] = false;
            } else {
                self.state.bottom_row[GAME_WIDTH - 1] = false;
            }
        }

        if self.is_collision() {
            self.state.is_game_over = true;
        }
    }

    pub fn handle_input(&mut self, movement: PlayerMove) {
        if self.state.is_game_over {
            return;
        }

        match movement {
            PlayerMove::Up => self.state.player_pos.1 = 0,
            PlayerMove::Down => self.state.player_pos.1 = 1,
            PlayerMove::Quit => self.state.is_game_over = true,
        }
    }

    fn is_collision(&self) -> bool {
        let (x, y) = self.state.player_pos;
        (y == 0 && self.state.top_row[x]) || (y == 1 && self.state.bottom_row[x])
    }

    pub fn handle_game_over(&self) -> Vec<(String, u32, Option<u64>)> {
        if !self.state.is_game_over {
            return vec![];
        }
        
        self.score_manager.handle_new_score(self.state.score)
    }
} 
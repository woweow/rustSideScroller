use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::{MoveTo},
};

pub struct GameRenderer;

impl GameRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, score: u32, top_row: &[bool], bottom_row: &[bool], player_pos: (usize, usize)) {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        
        println!("Score: {}", score);
        
        execute!(io::stdout(), MoveTo(0, 1)).unwrap();
        
        for (i, &has_obstacle) in top_row.iter().enumerate() {
            if player_pos == (i, 0) {
                print!("x");
            } else if has_obstacle {
                print!("-");
            } else {
                print!(" ");
            }
        }
        
        execute!(io::stdout(), MoveTo(0, 2)).unwrap();
        
        for (i, &has_obstacle) in bottom_row.iter().enumerate() {
            if player_pos == (i, 1) {
                print!("x");
            } else if has_obstacle {
                print!("-");
            } else {
                print!(" ");
            }
        }
        
        io::stdout().flush().unwrap();
    }

    pub fn show_game_over(&self, score: u32) {
        println!("\nGame Over! Final score: {}", score);
    }
} 
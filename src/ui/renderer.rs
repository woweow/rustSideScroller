use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
};
use crate::core::GameState;

pub fn render_game(state: &GameState) {
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
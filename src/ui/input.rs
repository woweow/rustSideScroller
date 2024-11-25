use std::io::{self, Write};
use std::time::Duration;
use crossterm::event::{poll, read, Event, KeyCode};
use crate::core::PlayerMove;

pub fn handle_input(duration: Duration) -> Option<PlayerMove> {
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

pub fn ask_play_again() -> bool {
    print!("Play again? (y/n): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
} 
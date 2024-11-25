use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::{Reply, Rejection};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::core::{Game, PlayerMove};
use simple_kv_store::KvStore;

#[derive(Serialize)]
struct NewGameResponse {
    game_id: String,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    movement: String,
}

pub async fn new_game(
    store: Arc<Mutex<KvStore>>,
    games: Arc<Mutex<HashMap<String, Game>>>,
) -> Result<impl Reply, Rejection> {
    let game_id = Uuid::new_v4().to_string();
    let game = Game::new(store);
    
    games.lock().unwrap().insert(game_id.clone(), game);
    
    Ok(warp::reply::json(&NewGameResponse { game_id }))
}

pub async fn get_state(
    game_id: String,
    games: Arc<Mutex<HashMap<String, Game>>>,
) -> Result<impl Reply, Rejection> {
    let games = games.lock().unwrap();
    if let Some(game) = games.get(&game_id) {
        Ok(warp::reply::json(&game.get_state()))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn make_move(
    game_id: String,
    move_req: MoveRequest,
    games: Arc<Mutex<HashMap<String, Game>>>,
) -> Result<impl Reply, Rejection> {
    let mut games = games.lock().unwrap();
    
    if let Some(game) = games.get_mut(&game_id) {
        let movement = match move_req.movement.as_str() {
            "up" => PlayerMove::Up,
            "down" => PlayerMove::Down,
            "quit" => PlayerMove::Quit,
            _ => return Err(warp::reject::custom(InvalidMove)),
        };
        
        game.handle_input(movement);
        game.update();
        
        Ok(warp::reply::json(&game.get_state()))
    } else {
        Err(warp::reject::not_found())
    }
}

#[derive(Debug)]
struct InvalidMove;
impl warp::reject::Reject for InvalidMove {} 
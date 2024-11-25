use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::Filter;
use simple_kv_store::KvStore;
use crate::core::Game;
use crate::server::handlers;

pub struct GameServer {
    store: Arc<Mutex<KvStore>>,
    games: Arc<Mutex<HashMap<String, Game>>>,
}

impl GameServer {
    pub fn new(store: Arc<Mutex<KvStore>>) -> Self {
        Self {
            store,
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self, port: u16) {
        let games = self.games.clone();
        let store = self.store.clone();

        // Routes
        let new_game = warp::post()
            .and(warp::path("game"))
            .and(warp::path("new"))
            .and(with_store(store.clone()))
            .and(with_games(games.clone()))
            .and_then(handlers::new_game);

        let get_state = warp::get()
            .and(warp::path("game"))
            .and(warp::path::param())
            .and(with_games(games.clone()))
            .and_then(handlers::get_state);

        let make_move = warp::post()
            .and(warp::path("game"))
            .and(warp::path::param())
            .and(warp::path("move"))
            .and(warp::body::json())
            .and(with_games(games.clone()))
            .and_then(handlers::make_move);

        let routes = new_game
            .or(get_state)
            .or(make_move)
            .with(warp::cors().allow_any_origin());

        println!("Game server running on http://localhost:{}", port);
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }
}

fn with_store(
    store: Arc<Mutex<KvStore>>,
) -> impl Filter<Extract = (Arc<Mutex<KvStore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

fn with_games(
    games: Arc<Mutex<HashMap<String, Game>>>,
) -> impl Filter<Extract = (Arc<Mutex<HashMap<String, Game>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || games.clone())
} 
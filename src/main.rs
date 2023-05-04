use crate::game::Game;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

mod game;

/// Wrapper around a game to handle the websocket side of things etc
#[derive(Debug)]
struct GameHandler {
    game: Game,
    events: broadcast::Sender<String>,
}

impl GameHandler {
    fn new(game: Game) -> Self {
        let (events, _rx) = broadcast::channel(100);
        GameHandler { game, events }
    }
}

#[derive(Clone)]
struct AppState {
    game_handlers: Arc<Mutex<HashMap<String, GameHandler>>>,
}

#[tokio::main]
async fn main() {
    // hardcode a game
    let mut handlers = HashMap::<String, GameHandler>::new();
    let id = "f1ea207c-e151-4a1d-a8ef-8f96a44fdff5";
    let mut game = Game::new(11, 18);
    game.add_player("Tim", "cookie1");
    game.add_player("Avi", "cookie2");
    handlers.insert(id.to_string(), GameHandler::new(game));

    let app_state = AppState {
        game_handlers: Arc::new(Mutex::new(handlers)),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/game", post(new_game))
        .with_state(app_state.clone())
        .route("/game", get(list_games))
        .with_state(app_state.clone())
        .route("/game/:id", get(get_game))
        .with_state(app_state.clone())
        .route("/game/:id/bomb", post(bomb))
        .with_state(app_state.clone())
        .route("/game/:id/ws", get(websocket_handler))
        .with_state(app_state.clone());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn new_game(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
    let mut handlers = state.game_handlers.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let game = Game::new(11, 18);
    handlers.insert(id.clone(), GameHandler::new(game));

    let updated_jar = jar.add(Cookie::new("battleship-id", "mycookie"));
    // .remove(Cookie::named("some-cookie"));

    (updated_jar, Json(json!({ "id": id })))
}

async fn list_games(State(state): State<AppState>) -> impl IntoResponse {
    let handlers = state.game_handlers.lock().unwrap();
    Json(handlers.keys().cloned().collect::<Vec<String>>())
}

async fn get_game(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let handlers = state.game_handlers.lock().unwrap();
    if let Some(handler) = handlers.get(&id) {
        Ok(Json(handler.game.clone()))
    } else {
        Err((StatusCode::NOT_FOUND, Json("not found")))
    }
}

async fn bomb(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<BombPayload>
) -> impl IntoResponse {
    let mut handlers = state.game_handlers.lock().unwrap();
    if let Some(handler) = handlers.get_mut(&id) {
        handler.game.bomb(payload.x, payload.y);
        let _ = handler.events.send("bombed".to_string());
        Ok(Json("ok"))
    } else {
        Err((StatusCode::NOT_FOUND, Json("not found")))
    }
}

async fn websocket_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let handlers = state.game_handlers.lock().unwrap();
    if let Some(handler) = handlers.get(&id) {
        let events = handler.events.subscribe();
        let ws_upgrade = ws.on_upgrade(|socket| websocket(socket, events));
        Ok(ws_upgrade)
    } else {
        Err((StatusCode::NOT_FOUND, Json("not found")))
    }
}

async fn websocket(stream: WebSocket, mut events: Receiver<String>) {
    let (mut sender, mut _receiver) = stream.split();

    while let Ok(event) = events.recv().await {
        if let Err(e) = sender.send(Message::Text(event)).await {
            println!("Got err sending to ws: {}", e);
            break;
        }
    }
}

#[derive(Deserialize)]
pub struct BombPayload {
    x: usize,
    y: usize,
}

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::game::Game;

use axum::{routing::get, routing::post, Router, Json};
use axum::extract::{Path, State};
use axum::extract::ws::close_code::STATUS;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tokio::sync::broadcast;
use serde_json::json;
use ticker::Ticker;
use uuid::Uuid;
use futures::{sink::SinkExt, stream::StreamExt};

mod game;

#[tokio::main]
async fn main() {
    let games = Arc::new(Mutex::new(HashMap::<String, Game>::new()));
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world)).with_state(games.clone()).
        route("/game", post(new_game)).with_state(games.clone()).
        route("/game", get(list_games)).with_state(games.clone()).
        route("/game/:id", get(get_game)).with_state(games.clone()).
        route("/game/ws", get(websocket_handler)).with_state(games.clone());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world(State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    games.insert(id, Game::new(10,10));
    Json(games.clone())
}

async fn new_game(State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    games.insert(id.clone(), Game::new(10,10));
    Json(json!({"id": id}))
}

async fn list_games(State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    // games.iter().map(|(id, _)| id).collect();
    Json(games.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>())
}

async fn get_game(Path(id): Path<String>, State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    let game = games.get(&id);
    if let Some(game) = games.get(&id) {
        Ok(Json(game.clone()))
    }else {
        Err((StatusCode::NOT_FOUND, Json("not found")))
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<HashMap<String,Game>>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<Mutex<HashMap<String,Game>>>) {
    let (mut sender, mut _receiver) = stream.split();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        for _ in Ticker::new((0..), Duration::from_secs(1)) {
            // In any websocket error, break loop.
            if sender.send(Message::Text("Hello World \n".parse().unwrap())).await.is_err() {
                break;
            }
        }
    });

}
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
use axum_extra::extract::cookie::{CookieJar, Cookie};

mod game;

#[tokio::main]
async fn main() {
    // hardcode a game
    let id = "f1ea207c-e151-4a1d-a8ef-8f96a44fdff5";
    let games = Arc::new(Mutex::new(HashMap::<String, Game>::new()));
        {
            let mut g = games.lock().unwrap();
            let mut game = Game::new(11, 18);

            Game::add_player(&mut game, "Tim", "cookie1");
            Game::add_player(&mut game, "Avi", "cookie2");
            g.insert(id.clone().parse().unwrap(), game);
        }

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

async fn new_game(jar: CookieJar, State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    games.insert(id.clone(), Game::new(10,10));

    let updated_jar = jar
        .add(Cookie::new("battleship-id", "mycookie"));
        // .remove(Cookie::named("some-cookie"));

    (updated_jar, Json(json!({"id": id})))
}

async fn list_games(State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
    Json(games.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>())
}

async fn get_game(Path(id): Path<String>, State(state): State<Arc<Mutex<HashMap<String,Game>>>>) -> impl IntoResponse {
    let mut games = state.lock().unwrap();
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

    // let mut send_task = tokio::spawn(async move {
        for _ in Ticker::new((0..), Duration::from_secs(1)) {
            // In any websocket error, break loop.
            if sender.send(Message::Text("Hello World \n".parse().unwrap())).await.is_err() {
                println!("disconnected");
                break;
            }
        }
    // });
}
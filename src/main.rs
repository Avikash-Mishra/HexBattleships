use crate::game::Game;

mod game;

fn main() {
    let mut game = Game::new(10, 10);
    game.add_player("Tim", "123");
    println!("{}", serde_json::to_string(&game).unwrap());
}

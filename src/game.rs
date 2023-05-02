use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Player {
    name: String,
    cookie: String
}

#[derive(Debug, Serialize)]
pub struct PlayerIdx(usize);

#[derive(Debug, Serialize)]
pub struct Game {
    players: Vec<Player>,
    board: Board,
    state: GameState
}

#[derive(Debug, Serialize)]
pub enum GameState {
    WaitingForPlayers,
    Playing(PlayingState),
    Finished(FinishedState)
}

#[derive(Debug, Serialize)]
pub enum Action {
    Start,
    AddPlayer(String, String),
    PlayTurn
}

#[derive(Debug, Serialize)]
pub struct PlayingState {
    next_turn: PlayerIdx,
    die_order: Vec<PlayerIdx>
}

#[derive(Debug, Serialize)]
pub struct FinishedState {
    die_order: Vec<PlayerIdx>
}

#[derive(Debug, Serialize, Clone)]
pub struct Board {
    height: i32,
    width: i32,
    board: Vec<Vec<BoardCell>>
}

#[derive(Debug, Serialize, Clone)]
pub struct BoardCell {
    uncovered: bool,
    ship_nodes: Vec<PlayerIdx>
}

impl Game {
    pub fn new(height: i32, width: i32) -> Self {
        Game {
            players: vec![],
            board: Board::new(height, width),
            state: GameState::WaitingForPlayers,
        }
    }

    pub fn add_player(&mut self, name: &str, cookie: &str) {
        if let GameState::WaitingForPlayers = &self.state {
            self.players.push(Player {
                name: name.to_string(),
                cookie: cookie.to_string(),
            })
        } else {
            println!("Tried to add new player while not in the waiting for players state");
        }
    }
    
    pub fn start_game(&mut self) {
        if let GameState::WaitingForPlayers = &self.state {
            if self.players.len() >= 2 {
                self.state = GameState::Playing(PlayingState {
                    next_turn: PlayerIdx(0),
                    die_order: vec![],
                })
            }
        } else {
            println!("Tried to start the game when its already been started");
        }
    }
}

impl Board {
    fn new(height: i32, width: i32) -> Self {
        let board = (0..width).into_iter().map(|_| {
            (0..height).into_iter().map(|_| BoardCell::new()).collect()
        }).collect();

        Board {
            height,
            width,
            board,
        }
    }
}

impl BoardCell {
    fn new() -> Self {
        BoardCell {
            uncovered: false,
            ship_nodes: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::*;

    #[test]
    fn test_new_game() {
        let mut game = Game::new(10, 5);
        game.add_player("Tim", "1234");
        assert_eq!(game.board.height, 10);
    }
}

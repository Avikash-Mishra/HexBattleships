use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Player {
    name: String,
    cookie: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerIdx(usize);

#[derive(Debug, Serialize, Clone)]
pub struct Game {
    players: Vec<Player>,
    board: Board,
    state: GameState,
}

#[derive(Debug, Serialize, Clone)]
pub enum GameState {
    WaitingForPlayers,
    Playing(PlayingState),
    Finished(FinishedState),
}

#[derive(Debug, Serialize, Clone)]
pub enum Action {
    Start,
    AddPlayer(String, String),
    PlayTurn,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayingState {
    next_turn: PlayerIdx,
    die_order: Vec<PlayerIdx>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FinishedState {
    die_order: Vec<PlayerIdx>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Board {
    height: i32,
    width: i32,
    data: Vec<Vec<BoardCell>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BoardCell {
    uncovered: bool,
    hits: Vec<PlayerIdx>,
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
            });
            self.board.data[1][1].hits.push(PlayerIdx(self.players.len()-1));
            self.board.data[2][self.players.len()].hits.push(PlayerIdx(self.players.len()-1))
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
        let board = (0..height)
            .into_iter()
            .map(|_| (0..width).into_iter().map(|_| BoardCell::new()).collect())
            .collect();

        Board {
            height,
            width,
            data: board,
        }
    }
}

impl BoardCell {
    fn new() -> Self {
        BoardCell {
            uncovered: false,
            hits: vec![],
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

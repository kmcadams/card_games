use crate::player::Player;

pub(super) struct BlackjackPlayers {
    pub(super) player: Player,
    pub(super) dealer: Player,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    Player,
    Dealer,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq)]

pub enum PlayerAction {
    Hit,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameResult {
    Pending,
    PlayerWin,
    DealerWin,
    Push,
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Pending => write!(f, "⏳ Game in progress..."),
            GameResult::PlayerWin => write!(f, "🎉 You win!"),
            GameResult::DealerWin => write!(f, "💥 Dealer wins!"),
            GameResult::Push => write!(f, "🤝 Push!"),
        }
    }
}

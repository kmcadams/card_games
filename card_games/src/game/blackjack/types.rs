#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Dealing,
    PlayerTurn,
    DealerTurn,
    RoundOver,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Dealing => write!(f, "Dealing cards..."),
            Phase::PlayerTurn => write!(f, "Your turn"),
            Phase::DealerTurn => write!(f, "Dealer's turn"),
            Phase::RoundOver => write!(f, "Round over"),
        }
    }
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

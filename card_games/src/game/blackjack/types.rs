use crate::{bank::bet::Bet, cards::hand::Hand, player::Player};

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
#[derive(Debug, Clone)]
pub struct PlayerHand {
    pub hand: Hand,
    pub bet: Bet,
    pub is_complete: bool,
}

impl PlayerHand {
    pub fn new(bet_amount: u32) -> Self {
        PlayerHand {
            hand: Hand::new(),
            bet: Bet { amount: bet_amount },
            is_complete: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveHand {
    Primary,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
    Hit,
    Stay,
    Double,
    Split,
    NewRound,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameResult {
    Pending,
    PlayerWin,
    DealerWin,
    Push,
}

impl GameResult {
    pub fn determine(player: u8, dealer: u8) -> Self {
        match (player > 21, dealer > 21) {
            (true, _) => GameResult::DealerWin,
            (_, true) => GameResult::PlayerWin,
            _ if player > dealer => GameResult::PlayerWin,
            _ if dealer > player => GameResult::DealerWin,
            _ => GameResult::Push,
        }
    }
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

mod tests {
    use super::*;

    #[test]
    fn test_game_result_determine() {
        assert_eq!(GameResult::determine(21, 20), GameResult::PlayerWin);
        assert_eq!(GameResult::determine(20, 21), GameResult::DealerWin);
        assert_eq!(GameResult::determine(21, 21), GameResult::Push);
    }
}

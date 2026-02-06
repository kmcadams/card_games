use crate::{
    bank::bet::Bet,
    cards::{deck_builder::DeckBuilder, hand::Hand, Card, Deck},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackjackState {
    Dealing,
    PlayerTurn { hand_index: usize },
    DealerTurn,
    RoundOver,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shoe {
    deck: Deck,
}

impl Shoe {
    pub fn new_shuffled() -> Self {
        let mut deck = DeckBuilder::new().standard52().build();
        deck.shuffle();
        Self { deck }
    }

    pub fn remaining(&self) -> usize {
        self.deck.len()
    }

    pub fn draw(&mut self) -> Card {
        self.deck.draw().expect("Deck exhausted")
    }

    #[cfg(test)]
    pub fn rigged(draw_order: Vec<Card>) -> Self {
        Self {
            deck: Deck::from_cards(draw_order),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub player_hands: Vec<PlayerHand>,
    pub dealer_hand: Hand,
}

impl std::fmt::Display for BlackjackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlackjackState::Dealing => write!(f, "Dealing cards..."),
            BlackjackState::PlayerTurn { .. } => write!(f, "Your turn"),
            BlackjackState::DealerTurn => write!(f, "Dealer's turn"),
            BlackjackState::RoundOver => write!(f, "Round over"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Hit,
    Stay,
    Double,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_result_determine() {
        assert_eq!(GameResult::determine(21, 20), GameResult::PlayerWin);
        assert_eq!(GameResult::determine(20, 21), GameResult::DealerWin);
        assert_eq!(GameResult::determine(21, 21), GameResult::Push);
    }
}

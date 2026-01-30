use crate::{
    cards::Card,
    game::blackjack::{GameResult, Phase},
};

#[derive(Debug, Clone)]
pub struct BlackjackView {
    pub phase: Phase,

    pub player_cards: Vec<VisibleCard>,
    pub dealer_cards: Vec<VisibleCard>,

    pub player_score: u8,
    pub dealer_score: Option<u8>,

    pub result: GameResult,

    pub can_hit: bool,
    pub can_stay: bool,
}

#[derive(Clone, Debug)]
pub enum VisibleCard {
    FaceUp(Card),
    FaceDown,
}

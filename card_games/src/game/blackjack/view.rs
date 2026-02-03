use crate::{
    cards::Card,
    game::blackjack::{types::PlayerAction, GameResult, Phase},
};

#[derive(Debug, Clone)]
pub struct BlackjackView {
    pub available_actions: Vec<PlayerAction>,
    pub phase: Phase,

    // pub player_cards: Vec<VisibleCard>,
    pub player_hands: Vec<PlayerHandView>,
    pub active_hand_index: usize,
    pub dealer_cards: Vec<VisibleCard>,

    pub dealer_visible_score: Option<u8>,
    pub dealer_has_hidden_card: bool,

    pub bank_balance: u32,

    pub result: GameResult,

    pub can_hit: bool,
    pub can_stay: bool,
    pub can_start_new_round: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerHandView {
    pub cards: Vec<VisibleCard>,
    pub score: u8,
    pub bet_amount: u32,
    pub is_complete: bool,
}

#[derive(Clone, Debug)]
pub enum VisibleCard {
    FaceUp(Card),
    FaceDown,
}

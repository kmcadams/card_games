use crate::cards::Card;

pub trait Game {
    type Outcome;

    fn setup(&mut self);
    fn play_round(&mut self) -> Option<Self::Outcome>;
    fn is_finished(&self) -> bool;
    fn winner(&self) -> Option<String>;
}

pub enum GameState {
    Waiting,
    InProgress,
    Complete,
}

pub trait GameRules {
    fn card_value(card: &Card) -> u8;
}

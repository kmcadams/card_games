use crate::cards::Card;

pub trait Game {
    type Outcome;

    fn setup(&mut self);
    fn play(&mut self);
    fn is_finished(&self) -> bool;
    fn winner(&self) -> Self::Outcome;
}

pub enum GameState {
    Waiting,
    InProgress,
    Complete,
}

pub trait GameRules {
    fn card_value(card: &Card) -> u8;
}

use crate::cards::Card;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub trait Game {
    type Outcome;
    type Action;
    type View;

    fn setup(&mut self);
    fn play(&mut self);
    fn run(&mut self, tx: Sender<GameEvent<Self::View>>, rx: Receiver<Self::Action>);
    fn apply_action(&mut self, action: Self::Action);
    fn view(&self) -> Self::View;
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
#[derive(Debug)]
pub enum GameEvent<V> {
    ViewUpdated(V),
    RequestInput,
    GameOver,
}

use crate::cards::{hand::Hand, Card};
use crate::game::blackjack::{GameResult, BlackjackState};

pub trait BlackjackDisplay {
    fn show_turn(&mut self, phase: &BlackjackState);
    fn show_hand(&mut self, label: &str, hand: &Hand);
    fn show_score(&mut self, label: &str, score: u8);
    fn show_card_drawn(&mut self, card: &Card);
    fn show_result(&mut self, result: &GameResult);
    fn show_message(&mut self, message: &str);
}

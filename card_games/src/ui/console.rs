use crate::cards::{hand::Hand, Card};
use crate::game::blackjack::{GameResult, Phase};

use super::blackjack_display::BlackjackDisplay;

//TODO: Redesign Views
pub struct ConsoleDisplay;

impl BlackjackDisplay for ConsoleDisplay {
    fn show_turn(&mut self, turn: &Phase) {
        match turn {
            Phase::Dealing => println!("\n=== Dealing Cards ==="),
            Phase::PlayerTurn => println!("\n=== Your Turn ==="),
            Phase::DealerTurn => println!("\n=== Dealer's Turn ==="),
            Phase::RoundOver => println!("\n=== Game Over ==="),
        }
    }

    fn show_hand(&mut self, label: &str, hand: &Hand) {
        println!("{} hand: {}", label, hand);
    }

    fn show_score(&mut self, label: &str, score: u8) {
        println!("{} score: {}", label, score);
    }

    fn show_card_drawn(&mut self, card: &Card) {
        println!("Drew: {}", card);
    }

    fn show_result(&mut self, result: &GameResult) {
        println!("{}", result);
    }

    fn show_message(&mut self, message: &str) {
        println!("{}", message);
    }
}

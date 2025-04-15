use std::fmt::Display;

use super::card::Card;

#[derive(Debug, Clone)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Hand { cards: vec![] }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn clear_hand(&mut self) {
        self.cards.clear()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cards.iter().fold(Ok(()), |result, card| {
            result.and_then(|_| write!(f, "|{}|", card))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::{Suit, Value};

    #[test]
    fn hand_adds_and_clears_cards() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::ACE));
        assert_eq!(hand.cards().len(), 1);

        hand.clear_hand();
        assert!(hand.cards().is_empty());
    }

    #[test]
    fn hand_len_and_is_empty_work() {
        let mut hand = Hand::new();
        assert!(hand.is_empty());

        hand.add(Card::new(Suit::SPADES, Value::KING));
        assert_eq!(hand.len(), 1);
        assert!(!hand.is_empty());
    }

    #[test]
    fn hand_display_formats_all_cards() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::TEN));
        hand.add(Card::new(Suit::CLUBS, Value::ACE));
        let output = format!("{}", hand);
        assert!(output.contains("Ten of ♥"));
        assert!(output.contains("Ace of ♣"));
    }
}

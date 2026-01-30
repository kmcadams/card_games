//! A player's hand of cards, typically used in card games like Blackjack or Poker.
//!
//! This struct provides utility methods for managing and displaying a hand of cards,
//! such as adding, clearing, inspecting, and printing the cards.
use std::fmt::Display;

use super::card::Card;

/// Represents a hand of playing cards.

#[derive(Debug, Clone)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    /// Creates a new, empty hand.
    ///
    /// # Example
    /// ```
    /// use card_games::cards::hand::Hand;
    /// let hand = Hand::new();
    /// assert!(hand.is_empty());
    /// ```
    pub fn new() -> Self {
        Hand { cards: vec![] }
    }

    /// Adds a card to the hand.
    ///
    /// # Example
    /// ```
    /// use card_games::cards::{Card,Suit,Value, hand::Hand};
    /// let mut hand = Hand::new();
    /// hand.add(Card::new(Suit::SPADES, Value::ACE));
    /// ```
    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Clears all cards from the hand.

    pub fn clear_hand(&mut self) {
        self.cards.clear()
    }

    /// Returns the number of cards in the hand.

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns `true` if the hand contains no cards.

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Returns a slice of the cards currently in the hand.

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

impl std::ops::Deref for Hand {
    type Target = [Card];

    fn deref(&self) -> &Self::Target {
        &self.cards
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

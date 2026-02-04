//! Represents a deck of cards and provides functionality to shuffle, deal, and draw.
//!
//! The `Deck` struct is typically created using the [`DeckBuilder`](crate::cards::deck_builder::DeckBuilder),
//! allowing for flexible composition of custom or standard decks.

use super::card::Card;
use std::fmt::Display;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::player::player::Player;

/// Alias for identifying players when dealing cards.
pub type PlayerId = u8;

/// A collection of cards with functionality for shuffling, drawing, and dealing.
#[derive(Debug, Clone, PartialEq)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a deck from a vector of cards.
    pub fn from_cards(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    /// Creates a new deck using a set of cards.

    pub fn new(cards: Vec<Card>) -> Self {
        Self::from_cards(cards)
    }

    /// Returns the number of remaining cards in the deck.

    pub fn remaining_cards(&self) -> usize {
        self.cards.len()
    }

    /// Shuffles the cards in the deck.

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }
    /// Deals a number of cards to the provided players.
    ///
    /// # Errors
    /// Returns an error if the deck runs out of cards.
    pub fn deal<'a, I>(&mut self, num_to_deal: u8, players: I) -> Result<(), String>
    where
        I: IntoIterator<Item = &'a mut Player>,
    {
        let mut players: Vec<&'a mut Player> = players.into_iter().collect();

        for _ in 0..num_to_deal {
            for player in players.iter_mut() {
                if let Some(card) = self.cards.pop() {
                    player.hand.add(card);
                } else {
                    return Err("Deck is out of cards!".into());
                }
            }
        }

        Ok(())
    }

    /// Draws a single card from the top of the deck.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Returns a reference to the top card without removing it.
    pub fn peek(&self) -> Option<&Card> {
        self.cards.last()
    }
    /// Adds a card to the bottom of the deck.
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }
    /// Checks if a card is contained in the deck.
    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    /// Returns `true` if the deck has no remaining cards.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Returns the number of cards in the deck.
    pub fn len(&self) -> usize {
        self.cards.len()
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cards.iter().fold(Ok(()), |result, card| {
            result.and_then(|_| write!(f, "|{}|", card))
        })
    }
}

impl IntoIterator for Deck {
    type Item = Card;
    type IntoIter = std::vec::IntoIter<Card>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}
#[cfg(test)]
mod tests {
    use crate::cards::deck_builder::DeckBuilder;

    #[test]
    fn deck_new_standard52_has_52_cards() {
        let deck = DeckBuilder::new().standard52().build();
        assert_eq!(deck.remaining_cards(), 52);
    }

    #[test]
    fn deck_draw_removes_card() {
        let mut deck = DeckBuilder::new().standard52().build();
        let initial = deck.remaining_cards();
        let card = deck.draw();
        assert!(card.is_some());
        assert_eq!(deck.remaining_cards(), initial - 1);
    }

    #[test]
    fn deck_peek_does_not_remove_card() {
        let deck = DeckBuilder::new().standard52().build();
        let top = deck.peek().cloned();
        assert_eq!(deck.remaining_cards(), 52);
        assert_eq!(deck.peek().cloned(), top);
    }

    #[test]
    fn deck_reset_with_builder_refills_deck() {
        let builder = DeckBuilder::new().standard52();
        let mut deck = builder.clone().build();
        deck.draw();
        assert!(deck.remaining_cards() < 52);

        deck = builder.build();
        assert_eq!(deck.remaining_cards(), 52);
    }
}

//! A fluent builder for constructing custom decks.
//!
//! Use [`DeckBuilder`] when you want to configure a deck with specific
//! characteristics such as including jokers or repeating the deck multiple times.

use super::{
    card::{Card, Suit, Value},
    Deck,
};

/// A fluent builder for constructing [`Deck`] instances with custom parameters.
#[derive(Debug, Default, Clone)]
pub struct DeckBuilder {
    base_cards: Vec<Card>,
    repeat_count: usize,
    include_jokers: bool,
}

impl DeckBuilder {
    /// Creates a new [`DeckBuilder`] instance with default settings.
    ///
    /// # Example
    /// ```
    /// use card_games::cards::deck_builder::DeckBuilder;
    /// let builder = DeckBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the builder with a standard 52-card deck.
    pub fn standard52(mut self) -> Self {
        self.base_cards = Suit::standard_suits()
            .flat_map(|suit| Value::standard_values().map(move |value| Card::new(suit, value)))
            .collect();
        self
    }

    /// Includes two jokers per repetition of the deck.
    pub fn with_jokers(mut self) -> Self {
        self.include_jokers = true;
        self
    }

    /// Repeats the deck multiple times (e.g., 2x = 104 cards).
    pub fn repeat(mut self, count: usize) -> Self {
        self.repeat_count = count;
        self
    }

    /// Builds and returns the configured [`Deck`].
    pub fn build(self) -> Deck {
        let mut cards = Vec::new();

        for _ in 0..self.repeat_count.max(1) {
            cards.extend(self.base_cards.iter().copied());

            if self.include_jokers {
                cards.push(Card::joker());
                cards.push(Card::joker());
            }
        }

        Deck::from_cards(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_standard_52() {
        let deck = DeckBuilder::new().standard52().build();
        assert_eq!(deck.remaining_cards(), 52);
        assert!(!deck.into_iter().any(|c| c.is_joker()));
    }

    #[test]
    fn build_with_jokers() {
        let deck = DeckBuilder::new().standard52().with_jokers().build();
        let jokers = deck.into_iter().filter(|c| c.is_joker()).count();
        assert_eq!(jokers, 2);
    }

    #[test]
    fn build_repeated_deck() {
        let deck = DeckBuilder::new().standard52().repeat(2).build();
        assert_eq!(deck.remaining_cards(), 104);
    }

    #[test]
    fn build_repeated_with_jokers() {
        let deck = DeckBuilder::new()
            .standard52()
            .with_jokers()
            .repeat(3)
            .build();
        let jokers = deck.into_iter().filter(|c| c.is_joker()).count();
        assert_eq!(jokers, 6);
    }
}

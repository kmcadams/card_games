//! Types and utilities for representing and working with playing cards.
//!
//! Includes [`Suit`], [`Value`], and [`Card`] structs and helpers for building
//! decks, checking card properties, and game-specific logic.

use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Represents the suit of a playing card.
///
/// Includes standard suits (`Clubs`, `Diamonds`, `Hearts`, `Spades`) and a special `Joker` variant.
#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
pub enum Suit {
    CLUBS,
    DIAMONDS,
    HEARTS,
    SPADES,
    JOKER,
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Suit::CLUBS => write!(f, "♣"),
            Suit::DIAMONDS => write!(f, "♦"),
            Suit::HEARTS => write!(f, "♥"),
            Suit::SPADES => write!(f, "♠"),
            Suit::JOKER => write!(f, "Joker"),
        }
    }
}

impl Suit {
    /// Returns an iterator over the four standard suits (excluding Joker).
    pub fn standard_suits() -> impl Iterator<Item = Suit> {
        Suit::iter().filter(|s| *s != Suit::JOKER)
    }

    /// Returns `true` if the suit is red (Hearts or Diamonds).
    pub fn is_red(&self) -> bool {
        matches!(self, Suit::DIAMONDS | Suit::HEARTS)
    }

    /// Returns `true` if the suit is black (Clubs or Spades).
    pub fn is_black(&self) -> bool {
        matches!(self, Suit::CLUBS | Suit::SPADES)
    }
}

/// Represents the face value of a playing card.
///
/// Includes numbered cards, face cards, Ace, and a Joker.
#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
#[repr(u8)]
pub enum Value {
    ACE = 1,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    TEN,
    JACK,
    QUEEN,
    KING,
    JOKER = 0,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Value::ACE => write!(f, "Ace"),
            Value::TWO => write!(f, "Two"),
            Value::THREE => write!(f, "Three"),
            Value::FOUR => write!(f, "Four"),
            Value::FIVE => write!(f, "Five"),
            Value::SIX => write!(f, "Six"),
            Value::SEVEN => write!(f, "Seven"),
            Value::EIGHT => write!(f, "Eight"),
            Value::NINE => write!(f, "Nine"),
            Value::TEN => write!(f, "Ten"),
            Value::JACK => write!(f, "Jack"),
            Value::QUEEN => write!(f, "Queen"),
            Value::KING => write!(f, "King"),
            Value::JOKER => write!(f, "Joker"),
        }
    }
}

impl Value {
    /// Returns an iterator over all standard card values (excluding Joker).
    pub fn standard_values() -> impl Iterator<Item = Value> {
        Value::iter().filter(|v| *v != Value::JOKER)
    }
    /// Returns `true` if the value is a face card (Jack, Queen, or King).
    pub fn is_face_card(&self) -> bool {
        matches!(self, Value::JACK | Value::QUEEN | Value::KING)
    }

    /// Returns `true` if the value is numeric (2 through 10).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Value::TWO
                | Value::THREE
                | Value::FOUR
                | Value::FIVE
                | Value::SIX
                | Value::SEVEN
                | Value::EIGHT
                | Value::NINE
                | Value::TEN
        )
    }
    /// Returns `true` if the value is an Ace.
    pub fn is_ace(&self) -> bool {
        matches!(self, Value::ACE)
    }

    /// Returns the numerical rank of the value (Ace = 1, King = 13, etc.).
    ///
    /// Returns `None` for Joker.
    pub fn rank(&self) -> Option<u8> {
        match self {
            Value::JOKER => None,
            _ => Some(*self as u8),
        }
    }
}

/// Represents a full playing card, consisting of a [`Suit`] and [`Value`].
///
/// Includes helpers for creating jokers, determining card color, face cards, and rank.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card {
    suit: Suit,
    value: Value,
}

impl Card {
    /// Constructs a new `Card` with the given suit and value.
    ///
    /// # Example
    /// ```
    /// use card_games::cards::{Card, Suit, Value};
    /// let card = Card::new(Suit::HEARTS, Value::ACE);
    /// ```
    pub fn new(suit: Suit, value: Value) -> Card {
        Card { suit, value }
    }
    /// Returns a reference to the card's value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Returns a reference to the card's suit.
    pub fn suit(&self) -> &Suit {
        &self.suit
    }

    /// Returns a Joker card.
    pub fn joker() -> Self {
        Self {
            suit: Suit::JOKER,
            value: Value::JOKER,
        }
    }

    /// Returns `true` if the card is a Joker.
    pub fn is_joker(&self) -> bool {
        self.value == Value::JOKER
    }

    /// Returns `true` if the card is a face card.
    pub fn is_face_card(&self) -> bool {
        self.value.is_face_card()
    }

    /// Returns `true` if the card is red (Hearts or Diamonds).
    pub fn is_red(&self) -> bool {
        self.suit.is_red()
    }

    /// Returns `true` if the card is black (Clubs or Spades).
    pub fn is_black(&self) -> bool {
        self.suit.is_black()
    }

    /// Returns the rank of the card, or `None` if it's a Joker.
    pub fn rank(&self) -> Option<u8> {
        self.value.rank()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_joker() {
            write!(f, "Joker")
        } else {
            write!(f, "{} of {}", self.value, self.suit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn card_display_is_correct() {
        let card = Card::new(Suit::HEARTS, Value::TEN);
        assert_eq!(format!("{}", card), "Ten of ♥");
    }

    #[test]
    fn card_fields_are_set_correctly() {
        let card = Card::new(Suit::DIAMONDS, Value::QUEEN);
        assert_eq!(*card.suit(), Suit::DIAMONDS);
        assert_eq!(*card.value(), Value::QUEEN);
    }

    #[test]
    fn joker_card_is_detected() {
        let joker = Card::joker();
        assert!(joker.is_joker());
        assert_eq!(format!("{}", joker), "Joker");
    }
}

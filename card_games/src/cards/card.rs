use std::fmt::Display;

use strum_macros::EnumIter;

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

#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
pub enum Value {
    ACE,
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
    JOKER,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card {
    suit: Suit,
    value: Value,
}

impl Card {
    pub fn new(suit: Suit, value: Value) -> Card {
        Card { suit, value }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn suit(&self) -> &Suit {
        &self.suit
    }
    pub fn joker() -> Self {
        Self {
            suit: Suit::JOKER,
            value: Value::JOKER,
        }
    }

    pub fn is_joker(&self) -> bool {
        self.value == Value::JOKER
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

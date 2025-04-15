use std::fmt::Display;

use strum::IntoEnumIterator;
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

impl Suit {
    pub fn standard_suits() -> impl Iterator<Item = Suit> {
        Suit::iter().filter(|s| *s != Suit::JOKER)
    }
    pub fn is_red(&self) -> bool {
        matches!(self, Suit::DIAMONDS | Suit::HEARTS)
    }

    pub fn is_black(&self) -> bool {
        matches!(self, Suit::CLUBS | Suit::SPADES)
    }
}

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
    pub fn standard_values() -> impl Iterator<Item = Value> {
        Value::iter().filter(|v| *v != Value::JOKER)
    }
    pub fn is_face_card(&self) -> bool {
        matches!(self, Value::JACK | Value::QUEEN | Value::KING)
    }

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

    pub fn is_ace(&self) -> bool {
        matches!(self, Value::ACE)
    }

    pub fn rank(&self) -> Option<u8> {
        match self {
            Value::JOKER => None,
            _ => Some(*self as u8),
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
    pub fn is_face_card(&self) -> bool {
        self.value.is_face_card()
    }

    pub fn is_red(&self) -> bool {
        self.suit.is_red()
    }

    pub fn is_black(&self) -> bool {
        self.suit.is_black()
    }

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

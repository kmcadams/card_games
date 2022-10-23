use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
pub enum Suit{
    CLUBS,
    DIAMONDS,
    HEARTS,
    SPADES
}

impl Display for Suit{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Suit::CLUBS => write!(f, "♣"),
            Suit::DIAMONDS => write!(f, "♦"),
            Suit::HEARTS => write!(f, "♥"),
            Suit::SPADES => write!(f, "♠"),         
        }
}
}

#[derive(Debug, Copy, Clone,PartialEq, EnumIter)]
pub enum Value{
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
    KING
}

impl Display for Value{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Value::ACE => write!(f,"Ace"),
            Value::TWO => write!(f, "Two"),
            Value::THREE => write!(f,"Three"),
            Value::FOUR => write!(f, "Four"), 
            Value::FIVE => write!(f,"Five"),
            Value::SIX => write!(f, "Six"), 
            Value::SEVEN => write!(f,"Seven"),
            Value::EIGHT => write!(f, "Eight"),
            Value::NINE => write!(f,"Nine"),
            Value::TEN => write!(f, "Ten"), 
            Value::JACK => write!(f,"Jack"),
            Value::QUEEN => write!(f, "Queen"),
            Value::KING => write!(f, "King"),          
        }
}
}

#[derive(Debug, PartialEq)]
pub struct Card{
    suit: Suit,
    value: Value
}

impl Card{
    fn add_card(suit: Suit, value: Value) -> Card{
        Card{
            suit,
            value
        }
    }
}

impl Display for Card{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} of {}", self.value, self.suit)
    }
}

#[derive(Debug, PartialEq)]
pub struct Deck{
    cards: Vec<Card>
}

impl Deck{
    pub fn new() -> Deck{
        let mut cards = Vec::new();
        for v in Value::iter(){
            for s in Suit::iter(){
                cards.push(Card::add_card(s,v))                
            }
        }
        
        Deck{
            cards
        }
    }

    pub fn remaining_cards(&self) -> usize{
        self.cards.iter().count()
    }
}

impl Display for Deck{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cards.iter().fold(Ok(()), |result, card|{
            result.and_then(|_| write!(f,"|{}|", card))
        })
    }
}
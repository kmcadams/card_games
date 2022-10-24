use std::collections::HashMap;
use std::fmt::Display;

use rand::thread_rng;
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::player::player::Player;

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

#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn shuffle(&mut self){
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn deal(&mut self, num_to_deal: u8, players: &mut HashMap<u8,Player>){
        if players.len() <= 0{
            println!("No Players, no deal.");
            return ()
        }
        
        for _ in 0..num_to_deal{
            for player in players.values_mut(){
                if let Some(card) = self.cards.pop(){
                    player.hand.add(card)
                }
            }
            // for player in &players{
            //     if let Some(card) = self.cards.pop(){
            //         tmp
            //         player.hand.add(card.to_owned());
            //     };
            // }            
        }
    }
}

impl Display for Deck{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cards.iter().fold(Ok(()), |result, card|{
            result.and_then(|_| write!(f,"|{}|", card))
        })
    }
}
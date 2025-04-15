use std::fmt::Display;

use super::deck::Card;


#[derive(Debug,Clone)]
pub struct Hand{
    cards: Vec<Card>
}

impl Hand{
    pub fn new()-> Self{
        Hand { cards: vec![] }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn clear_hand(&mut self){
        self.cards.clear()
    }
}

impl Display for Hand{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cards.iter().fold(Ok(()), |result, card|{
            result.and_then(|_| write!(f,"|{}|", card))
        })
    }
}
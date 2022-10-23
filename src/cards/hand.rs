use std::fmt::Display;

use super::deck::Card;


pub struct Hand{
    hand: Vec<Card>
}

impl Hand{
    pub fn new()-> Self{
        Hand { hand: vec![] }
    }

    pub fn add(&mut self, card: Card) {
        self.hand.push(card);
    }
}

impl Display for Hand{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hand.iter().fold(Ok(()), |result, card|{
            result.and_then(|_| write!(f,"|{}|", card))
        })
    }
}
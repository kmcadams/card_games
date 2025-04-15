use super::card::Card;
use super::deck_type::DeckType;

use std::fmt::Display;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::player::player::Player;

pub type PlayerId = u8;

#[derive(Debug, PartialEq)]
pub struct Deck {
    deck_type: DeckType,
    cards: Vec<Card>,
}

impl Deck {
    pub(crate) fn from_cards(deck_type: DeckType, cards: Vec<Card>) -> Self {
        Self { deck_type, cards }
    }
    pub fn new(deck_type: DeckType) -> Deck {
        deck_type.build()
    }

    pub fn remaining_cards(&self) -> usize {
        self.cards.len()
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }
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

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn peek(&self) -> Option<&Card> {
        self.cards.last()
    }

    pub fn reset(&mut self) {
        *self = self.deck_type.clone().build();
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

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
    use super::*;

    #[test]
    fn deck_new_standard52_has_52_cards() {
        let deck = Deck::new(DeckType::Standard52);
        assert_eq!(deck.remaining_cards(), 52);
    }

    #[test]
    fn deck_draw_removes_card() {
        let mut deck = Deck::new(DeckType::Standard52);
        let initial = deck.remaining_cards();
        let card = deck.draw();
        assert!(card.is_some());
        assert_eq!(deck.remaining_cards(), initial - 1);
    }

    #[test]
    fn deck_peek_does_not_remove_card() {
        let deck = Deck::new(DeckType::Standard52);
        let top = deck.peek().cloned();
        assert_eq!(deck.remaining_cards(), 52);
        assert_eq!(deck.peek().cloned(), top);
    }

    #[test]
    fn deck_reset_refills_deck() {
        let mut deck = Deck::new(DeckType::Standard52);
        deck.draw();
        deck.reset();
        assert_eq!(deck.remaining_cards(), 52);
    }
}

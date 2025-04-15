use strum::IntoEnumIterator;

use super::{
    card::{Card, Suit, Value},
    Deck,
};

#[derive(Debug, Clone)]
pub enum DeckType {
    Standard52,
    WithJokers,
    Pinochle,
    Double52,
    Custom(Vec<Card>),
}
impl DeckType {
    pub fn build(self) -> Deck {
        match self {
            Self::Standard52 => {
                let cards = Suit::iter()
                    .filter(|suit| *suit != Suit::JOKER)
                    .flat_map(|suit| {
                        Value::iter()
                            .filter(|value| *value != Value::JOKER)
                            .map(move |value| Card::new(suit, value))
                    })
                    .collect();
                Deck::from_cards(cards)
            }

            Self::WithJokers => {
                let mut cards = Suit::iter()
                    .flat_map(|suit| Value::iter().map(move |value| Card::new(suit, value)))
                    .collect::<Vec<_>>();
                cards.push(Card::joker());
                cards.push(Card::joker());
                Deck::from_cards(cards)
            }

            Self::Double52 => {
                let cards = (0..2)
                    .flat_map(|_| {
                        Suit::iter()
                            .flat_map(|suit| Value::iter().map(move |value| Card::new(suit, value)))
                    })
                    .collect();
                Deck::from_cards(cards)
            }

            Self::Pinochle => {
                let relevant_values = [
                    Value::NINE,
                    Value::TEN,
                    Value::JACK,
                    Value::QUEEN,
                    Value::KING,
                    Value::ACE,
                ];
                let cards = (0..2)
                    .flat_map(|_| {
                        Suit::iter().flat_map(|suit| {
                            relevant_values
                                .iter()
                                .map(move |&value| Card::new(suit, value))
                        })
                    })
                    .collect();
                Deck::from_cards(cards)
            }

            Self::Custom(cards) => Deck::from_cards(cards),
        }
    }
}

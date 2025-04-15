use super::{
    card::{Card, Suit, Value},
    Deck,
};

#[derive(Debug, Clone, PartialEq)]
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
                let cards = Suit::standard_suits()
                    .flat_map(|suit| {
                        Value::standard_values().map(move |value| Card::new(suit, value))
                    })
                    .collect();

                Deck::from_cards(self.clone(), cards)
            }

            Self::WithJokers => {
                let mut cards = Suit::standard_suits()
                    .flat_map(|suit| {
                        Value::standard_values().map(move |value| Card::new(suit, value))
                    })
                    .collect::<Vec<_>>();

                cards.push(Card::joker());
                cards.push(Card::joker());

                Deck::from_cards(self.clone(), cards)
            }

            Self::Double52 => {
                let cards = (0..2)
                    .flat_map(|_| {
                        Suit::standard_suits().flat_map(|suit| {
                            Value::standard_values().map(move |value| Card::new(suit, value))
                        })
                    })
                    .collect();

                Deck::from_cards(self.clone(), cards)
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
                        Suit::standard_suits().flat_map(|suit| {
                            relevant_values
                                .iter()
                                .map(move |&value| Card::new(suit, value))
                        })
                    })
                    .collect();

                Deck::from_cards(self.clone(), cards)
            }

            DeckType::Custom(ref cards) => Deck::from_cards(self.clone(), cards.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard52_contains_no_jokers() {
        let deck = Deck::new(DeckType::Standard52);
        assert!(!deck.into_iter().any(|c| c.is_joker()));
    }

    #[test]
    fn with_jokers_includes_two_jokers() {
        let deck = Deck::new(DeckType::WithJokers);
        let jokers = deck.into_iter().filter(|c| c.is_joker()).count();
        assert_eq!(jokers, 2);
    }

    #[test]
    fn pinochle_has_48_cards() {
        let deck = Deck::new(DeckType::Pinochle);
        assert_eq!(deck.remaining_cards(), 48);
    }

    #[test]
    fn double52_has_104_cards() {
        let deck = Deck::new(DeckType::Double52);
        assert_eq!(deck.remaining_cards(), 104);
    }
}

use crate::cards::{Card, Value};

pub struct BlackjackRules;

impl BlackjackRules {
    pub fn card_score(card: &Card) -> u8 {
        match card.value() {
            Value::TWO => 2,
            Value::THREE => 3,
            Value::FOUR => 4,
            Value::FIVE => 5,
            Value::SIX => 6,
            Value::SEVEN => 7,
            Value::EIGHT => 8,
            Value::NINE => 9,
            Value::TEN | Value::JACK | Value::QUEEN | Value::KING => 10,
            Value::ACE => 11,
            Value::JOKER => 0,
        }
    }

    pub fn hand_score(hand: &[Card]) -> u8 {
        let mut score = 0;
        let mut ace_count = 0;

        for card in hand {
            let card_score = Self::card_score(card);
            score += card_score;
            if card.value().eq(&Value::ACE) {
                ace_count += 1;
            }
        }

        while score > 21 && ace_count > 0 {
            score -= 10;
            ace_count -= 1;
        }

        score
    }

    pub fn is_bust(hand: &[Card]) -> bool {
        Self::hand_score(hand) > 21
    }

    pub fn is_blackjack(hand: &[Card]) -> bool {
        hand.len() == 2 && Self::hand_score(hand) == 21
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::Suit;
    use crate::cards::hand::Hand;

    #[test]
    fn blackjack_scoring_with_ace_adjusts_correctly() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::DIAMONDS, Value::ACE));
        hand.add(Card::new(Suit::SPADES, Value::NINE));
        hand.add(Card::new(Suit::HEARTS, Value::NINE));

        assert_eq!(BlackjackRules::hand_score(hand.cards()), 19);
    }
}

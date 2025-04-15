use crate::{
    cards::{Card, Value},
    game::game::GameRules,
};

pub struct BlackjackRules;

impl GameRules for BlackjackRules {
    fn card_value(card: &Card) -> u8 {
        match *card.value() {
            Value::ACE => 11,
            Value::TWO => 2,
            Value::THREE => 3,
            Value::FOUR => 4,
            Value::FIVE => 5,
            Value::SIX => 6,
            Value::SEVEN => 7,
            Value::EIGHT => 8,
            Value::NINE => 9,
            Value::TEN | Value::JACK | Value::QUEEN | Value::KING => 10,
            Value::JOKER => 0,
        }
    }
}

impl BlackjackRules {
    pub fn hand_score(hand: &[Card]) -> u8 {
        let (mut score, mut ace_count) = hand.iter().fold((0, 0), |(acc, aces), card| {
            let value = Self::card_value(card);
            let is_ace = *card.value() == Value::ACE;
            (acc + value, aces + is_ace as u8)
        });

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

    #[test]
    fn blackjack_detects_blackjack_hand() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::ACE));
        hand.add(Card::new(Suit::HEARTS, Value::KING));

        assert!(BlackjackRules::is_blackjack(hand.cards()));
    }

    #[test]
    fn hand_with_multiple_aces_adjusts_correctly() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::ACE));
        hand.add(Card::new(Suit::SPADES, Value::ACE));
        hand.add(Card::new(Suit::CLUBS, Value::NINE));

        assert_eq!(BlackjackRules::hand_score(hand.cards()), 21);
    }

    #[test]
    fn hand_busts_without_aces() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::TEN));
        hand.add(Card::new(Suit::HEARTS, Value::KING));
        hand.add(Card::new(Suit::DIAMONDS, Value::FIVE));

        assert!(BlackjackRules::is_bust(hand.cards()));
    }

    #[test]
    fn ace_prevents_bust() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::TEN));
        hand.add(Card::new(Suit::HEARTS, Value::SIX));
        hand.add(Card::new(Suit::DIAMONDS, Value::ACE));

        assert_eq!(BlackjackRules::hand_score(hand.cards()), 17);
        assert!(!BlackjackRules::is_bust(hand.cards()));
    }

    #[test]
    fn hand_with_three_cards_equals_21() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::SEVEN));
        hand.add(Card::new(Suit::CLUBS, Value::SEVEN));
        hand.add(Card::new(Suit::SPADES, Value::SEVEN));

        assert_eq!(BlackjackRules::hand_score(hand.cards()), 21);
        assert!(!BlackjackRules::is_blackjack(hand.cards()));
    }

    #[test]
    fn two_cards_can_bust_and_not_be_blackjack() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::JACK));
        hand.add(Card::new(Suit::SPADES, Value::QUEEN));

        assert_eq!(BlackjackRules::hand_score(hand.cards()), 20);
        assert!(!BlackjackRules::is_blackjack(hand.cards()));
        assert!(!BlackjackRules::is_bust(hand.cards()));
    }
}

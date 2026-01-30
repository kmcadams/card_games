//! BlackjackRules contains logic specific to the game of Blackjack,
//! including card scoring rules, bust detection, and natural blackjack checks.
//!
//! These rules assume standard Blackjack behavior:
//! - Aces count as 11 or 1, depending on total hand score
//! - Face cards count as 10
//! - Blackjack is exactly 21 with 2 cards

use crate::cards::{Card, Value};

/// Blackjack-specific rules for scoring and win conditions.

pub struct BlackjackRules;

/// Returns the Blackjack value of a given card.
///
/// Face cards are worth 10, aces are worth 11, and jokers are worth 0.
///
/// This function does not apply Ace-adjustment logic — see [`BlackjackRules::hand_score`] for that.
/// # Example
/// ```
/// use card_games::cards::{Card, Suit, Value};
/// use card_games::game::blackjack::rules::BlackjackRules;
/// use crate::card_games::game::game::GameRules;
///
/// let card = Card::new(Suit::HEARTS, Value::QUEEN);
/// assert_eq!(BlackjackRules::card_value(&card), 10);
/// ```

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

/// Calculates the score of a Blackjack hand.
///
/// Aces are counted as 11 unless the total score exceeds 21,
/// in which case they are counted as 1.
///
/// # Example
/// ```
/// use card_games::cards::{Card, Suit, Value};
/// use card_games::game::blackjack::rules::BlackjackRules;
/// let hand = vec![
///     Card::new(Suit::HEARTS, Value::ACE),
///     Card::new(Suit::CLUBS, Value::NINE),
///     Card::new(Suit::SPADES, Value::NINE),
/// ];
/// assert_eq!(BlackjackRules::hand_score(&hand), 19);
/// ```
pub fn hand_score(hand: &[Card]) -> u8 {
    let (mut score, mut ace_count) = hand.iter().fold((0, 0), |(acc, aces), card| {
        let value = card_value(card);
        let is_ace = card.value().eq(&Value::ACE);
        (acc + value, aces + is_ace as u8)
    });

    while score > 21 && ace_count > 0 {
        score -= 10;
        ace_count -= 1;
    }

    score
}

pub fn dealer_should_hit(hand: &[Card]) -> bool {
    hand_score(hand) < 17
}

/// Returns `true` if the hand is a bust (score > 21).
///
/// # Example
/// ```
/// use card_games::cards::{Card, Suit, Value};
/// use card_games::game::blackjack::rules::BlackjackRules;
/// let hand = vec![
///     Card::new(Suit::SPADES, Value::TEN),
///     Card::new(Suit::HEARTS, Value::TEN),
///     Card::new(Suit::DIAMONDS, Value::THREE),
/// ];
/// assert!(BlackjackRules::is_bust(&hand));
/// ```
pub fn is_bust(hand: &[Card]) -> bool {
    hand_score(hand) > 21
}

/// Returns `true` if the hand is a natural blackjack
/// (exactly 2 cards totaling 21).
///
/// # Example
/// ```
/// use card_games::cards::{Card, Suit, Value};
/// use card_games::game::blackjack::rules::BlackjackRules;
/// let hand = vec![
///     Card::new(Suit::SPADES, Value::ACE),
///     Card::new(Suit::HEARTS, Value::KING),
/// ];
/// assert!(BlackjackRules::is_blackjack(&hand));
/// ```
pub fn is_blackjack(hand: &[Card]) -> bool {
    hand.len() == 2 && hand_score(hand) == 21
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

        assert_eq!(hand_score(hand.cards()), 19);
    }

    #[test]
    fn blackjack_detects_blackjack_hand() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::ACE));
        hand.add(Card::new(Suit::HEARTS, Value::KING));

        assert!(is_blackjack(hand.cards()));
    }

    #[test]
    fn hand_with_multiple_aces_adjusts_correctly() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::ACE));
        hand.add(Card::new(Suit::SPADES, Value::ACE));
        hand.add(Card::new(Suit::CLUBS, Value::NINE));

        assert_eq!(hand_score(hand.cards()), 21);
    }

    #[test]
    fn hand_busts_without_aces() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::TEN));
        hand.add(Card::new(Suit::HEARTS, Value::KING));
        hand.add(Card::new(Suit::DIAMONDS, Value::FIVE));

        assert!(is_bust(hand.cards()));
    }

    #[test]
    fn ace_prevents_bust() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::SPADES, Value::TEN));
        hand.add(Card::new(Suit::HEARTS, Value::SIX));
        hand.add(Card::new(Suit::DIAMONDS, Value::ACE));

        assert_eq!(hand_score(hand.cards()), 17);
        assert!(!is_bust(hand.cards()));
    }

    #[test]
    fn hand_with_three_cards_equals_21() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::SEVEN));
        hand.add(Card::new(Suit::CLUBS, Value::SEVEN));
        hand.add(Card::new(Suit::SPADES, Value::SEVEN));

        assert_eq!(hand_score(hand.cards()), 21);
        assert!(!is_blackjack(hand.cards()));
    }

    #[test]
    fn two_cards_can_bust_and_not_be_blackjack() {
        let mut hand = Hand::new();
        hand.add(Card::new(Suit::HEARTS, Value::JACK));
        hand.add(Card::new(Suit::SPADES, Value::QUEEN));

        assert_eq!(hand_score(hand.cards()), 20);
        assert!(!is_blackjack(hand.cards()));
        assert!(!is_bust(hand.cards()));
    }
}

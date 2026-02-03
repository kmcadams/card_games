use crate::{
    bank::bank::Bank,
    cards::{deck_builder::DeckBuilder, hand::Hand, Card, Deck},
    game::blackjack::{
        rules,
        types::{ActiveHand, Phase, PlayerAction, PlayerHand},
        view::{BlackjackView, PlayerHandView, VisibleCard},
        GameResult,
    },
};

pub struct Blackjack {
    phase: Phase,
    deck: Deck,
    player_hands: Vec<PlayerHand>,
    active_hand: ActiveHand,
    dealer_hand: Hand,
    bank: Bank,
    result: GameResult,
}
impl Blackjack {
    pub fn new() -> Self {
        let mut game = Blackjack {
            phase: Phase::Dealing,
            deck: DeckBuilder::new().standard52().build(),
            player_hands: vec![PlayerHand::new(10)],
            active_hand: ActiveHand::Primary,
            dealer_hand: Hand::new(),

            bank: Bank::new(1_000),

            result: GameResult::Pending,
        };

        game.new_round();
        game
    }

    fn deal_initial_cards(&mut self) {
        let card1 = self.draw_card();
        let card2 = self.draw_card();
        let dealer_card1 = self.draw_card();
        let dealer_card2 = self.draw_card();

        {
            let hand = self.active_hand_mut();
            hand.hand.add(card1);
            hand.hand.add(card2);
        }

        self.dealer_hand.add(dealer_card1);
        self.dealer_hand.add(dealer_card2);

        self.resolve_blackjack_or_continue();
    }

    fn resolve_blackjack_or_continue(&mut self) {
        let player = &self.player_hands[0];

        let player_blackjack = rules::is_blackjack(&player.hand);
        let dealer_blackjack = rules::is_blackjack(&self.dealer_hand);

        match (player_blackjack, dealer_blackjack) {
            (true, true) => {
                // Push
                self.bank.deposit(player.bet.amount);
                self.result = GameResult::Push;
                self.phase = Phase::RoundOver;
            }

            (true, false) => {
                // Player blackjack wins 3:2
                let payout = player.bet.amount + (player.bet.amount * 3 / 2);
                self.bank.deposit(payout);
                self.result = GameResult::PlayerWin;
                self.phase = Phase::RoundOver;
            }

            (false, true) => {
                // Dealer blackjack
                self.result = GameResult::DealerWin;
                self.phase = Phase::RoundOver;
            }

            (false, false) => {
                // Normal play continues
                self.phase = Phase::PlayerTurn;
            }
        }
    }

    fn active_hand_index(&self) -> usize {
        match self.active_hand {
            ActiveHand::Primary => 0,
            ActiveHand::Split => 1,
        }
    }

    fn active_hand_mut(&mut self) -> &mut PlayerHand {
        let idx = self.active_hand_index();
        &mut self.player_hands[idx]
    }

    fn advance_hand(&mut self) {
        self.active_hand = match self.active_hand {
            ActiveHand::Primary => ActiveHand::Split,
            ActiveHand::Split => ActiveHand::Split,
        };
    }
    pub fn new_round(&mut self) {
        self.player_hands = vec![PlayerHand::new(10)];
        self.active_hand = ActiveHand::Primary;

        self.dealer_hand.clear_hand();
        self.result = GameResult::Pending;

        self.phase = Phase::Dealing;

        self.deck = DeckBuilder::new().standard52().build();
        self.deck.shuffle();

        let bet = self.player_hands[0].bet.amount;

        if !self.bank.withdraw(bet) {
            //TODO: add state/view for "out of money"
            return;
        }

        self.deal_initial_cards();
    }
    pub fn apply(&mut self, action: PlayerAction) {
        match (self.phase, action) {
            (Phase::PlayerTurn, PlayerAction::Hit) => {
                let card = self.draw_card();
                let hand = self.active_hand_mut();

                hand.hand.add(card);

                if rules::is_bust(&hand.hand) {
                    hand.is_complete = true;
                    if self.player_hands.len() == 2 && self.active_hand == ActiveHand::Primary {
                        self.advance_hand();
                        return;
                    }
                    self.phase = Phase::RoundOver;
                    self.result = GameResult::DealerWin;
                }
            }

            (Phase::PlayerTurn, PlayerAction::Stay) => {
                self.active_hand_mut().is_complete = true;

                if self.player_hands.len() == 2 && self.active_hand == ActiveHand::Primary {
                    self.advance_hand();
                    return;
                }

                self.phase = Phase::DealerTurn;
                self.play_dealer();
            }

            // (Phase::PlayerTurn, PlayerAction::Stay) => {
            //     self.active_hand_mut().is_complete = true;
            //     self.phase = Phase::DealerTurn;
            //     self.play_dealer();
            // }
            (Phase::RoundOver, PlayerAction::NewRound) => {
                self.new_round();
            }

            (Phase::PlayerTurn, PlayerAction::Double) => {
                let bet = {
                    let hand = self.active_hand_mut();
                    if !rules::can_double(&hand.hand) {
                        return;
                    }
                    hand.bet.amount
                };

                if !self.bank.withdraw(bet) {
                    return;
                }

                let card = self.draw_card();
                let hand = self.active_hand_mut();

                hand.bet.amount *= 2;
                hand.hand.add(card);
                hand.is_complete = true;

                self.phase = Phase::DealerTurn;
                self.play_dealer();
            }

            (Phase::PlayerTurn, PlayerAction::Split) => {
                let can_split = {
                    let hand = self.active_hand_mut();
                    rules::can_split(&hand.hand)
                };
                if !can_split {
                    return;
                }

                let bet = self.player_hands[0].bet.amount;
                if !self.bank.withdraw(bet) {
                    return;
                }

                let (c0, c1) = {
                    let original = &mut self.player_hands[0].hand;
                    let cards = original.cards().to_vec();
                    original.clear_hand();
                    (cards[0], cards[1])
                };

                let new_primary = self.draw_card();
                let new_split = self.draw_card();

                {
                    let original = &mut self.player_hands[0].hand;
                    original.add(c0);
                    original.add(new_primary);
                }

                let mut split_hand = PlayerHand::new(bet);
                split_hand.hand.add(c1);
                split_hand.hand.add(new_split);

                self.player_hands.push(split_hand);
                self.active_hand = ActiveHand::Primary;
            }

            _ => {
                // explicitly ignored
                // TODO, should update user to input an action?
            }
        }
    }

    fn play_dealer(&mut self) {
        while rules::dealer_should_hit(&self.dealer_hand) {
            let card = self.draw_card();
            self.dealer_hand.add(card);
        }

        self.resolve_round();
    }

    fn resolve_round(&mut self) {
        let dealer_score = rules::hand_score(&self.dealer_hand);
        let dealer_bust = rules::is_bust(&self.dealer_hand);

        // You currently store only one GameResult; we’ll set it to “best/worst”
        // for now so UI has something to show. You can upgrade later.
        let mut any_win = false;
        let mut any_push = false;
        let mut any_loss = false;

        for hand in &self.player_hands {
            let player_score = rules::hand_score(&hand.hand);
            let player_bust = rules::is_bust(&hand.hand);

            let result = if player_bust {
                GameResult::DealerWin
            } else if dealer_bust {
                GameResult::PlayerWin
            } else {
                GameResult::determine(player_score, dealer_score)
            };

            match result {
                GameResult::PlayerWin => {
                    self.bank.deposit(hand.bet.amount * 2);
                    any_win = true;
                }
                GameResult::Push => {
                    self.bank.deposit(hand.bet.amount);
                    any_push = true;
                }
                GameResult::DealerWin => {
                    any_loss = true;
                }
                _ => {}
            }
        }

        // Collapse multiple results into one display result for now
        self.result = if any_win && !any_loss {
            GameResult::PlayerWin
        } else if any_loss && !any_win && !any_push {
            GameResult::DealerWin
        } else if any_push && !any_win && !any_loss {
            GameResult::Push
        } else if any_win && any_loss {
            // Mixed outcome; pick something neutral-ish
            GameResult::Push
        } else {
            GameResult::Pending
        };

        self.phase = Phase::RoundOver;
    }

    fn draw_card(&mut self) -> Card {
        self.deck
            .draw()
            .expect("Deck exhausted during Blackjack round") //TODO: remove expect
    }
    pub fn view(&self) -> BlackjackView {
        let active_hand_index = self.active_hand_index();

        // Player hands (all face-up)
        let player_hands = self
            .player_hands
            .iter()
            .map(|h| PlayerHandView {
                cards: h.hand.iter().cloned().map(VisibleCard::FaceUp).collect(),
                score: rules::hand_score(&h.hand),
                bet_amount: h.bet.amount,
                is_complete: h.is_complete,
            })
            .collect::<Vec<_>>();

        // Dealer cards depend on phase
        let (dealer_cards, dealer_visible_score, dealer_has_hidden_card) = match self.phase {
            Phase::Dealing | Phase::PlayerTurn => {
                let visible_cards: Vec<Card> = self
                    .dealer_hand
                    .iter()
                    .skip(1) // skip hole card
                    .cloned()
                    .collect();

                let visible_score = if visible_cards.is_empty() {
                    None
                } else {
                    Some(rules::hand_score(&visible_cards))
                };

                let cards = self
                    .dealer_hand
                    .iter()
                    .enumerate()
                    .map(|(i, card)| {
                        if i == 0 {
                            VisibleCard::FaceDown
                        } else {
                            VisibleCard::FaceUp(card.clone())
                        }
                    })
                    .collect();

                (cards, visible_score, true)
            }

            Phase::DealerTurn | Phase::RoundOver => {
                let cards: Vec<VisibleCard> = self
                    .dealer_hand
                    .iter()
                    .cloned()
                    .map(VisibleCard::FaceUp)
                    .collect();

                let score = Some(rules::hand_score(&self.dealer_hand));

                (cards, score, false)
            }
        };

        let active_hand = &self.player_hands[active_hand_index];

        let mut controls = vec![PlayerAction::Quit];

        if self.phase == Phase::PlayerTurn {
            controls.insert(0, PlayerAction::Stay);
            controls.insert(0, PlayerAction::Hit);

            if rules::can_double(&active_hand.hand) && self.bank.balance() >= active_hand.bet.amount
            {
                controls.insert(0, PlayerAction::Double);
            }

            if self.player_hands.len() == 1
                && rules::can_split(&active_hand.hand)
                && self.bank.balance() >= active_hand.bet.amount
            {
                controls.insert(0, PlayerAction::Split);
            }
        }

        if self.phase == Phase::RoundOver {
            controls.insert(0, PlayerAction::NewRound);
        }

        BlackjackView {
            available_actions: controls,
            phase: self.phase,

            player_hands,
            active_hand_index,

            dealer_cards,
            dealer_visible_score,
            dealer_has_hidden_card,

            bank_balance: self.bank.balance(),

            result: self.result,

            can_hit: self.phase == Phase::PlayerTurn,
            can_stay: self.phase == Phase::PlayerTurn,
            can_start_new_round: self.phase == Phase::RoundOver,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Suit, Value};

    use super::*;

    #[test]
    fn new_round_withdraws_initial_bet() {
        let game = Blackjack::new();
        let view = game.view();

        assert_eq!(view.bank_balance, 1_000 - 10);
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn new_round_deals_initial_cards() {
        let game = Blackjack::new();
        let view = game.view();

        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.dealer_cards.len(), 2);
    }

    #[test]
    fn hit_adds_one_card_to_player_hand() {
        let mut game = Blackjack::new();

        let initial_cards = game.view().player_hands[0].cards.len();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.player_hands[0].cards.len(), initial_cards + 1);
    }

    #[test]
    fn stay_advances_to_round_over() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, Phase::RoundOver);
    }

    #[test]
    fn double_is_available_on_first_player_turn() {
        let game = Blackjack::new();
        let view = game.view();

        assert!(view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_is_not_available_after_hit() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert!(!view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_doubles_bet_and_withdraws_balance() {
        let mut game = Blackjack::new();

        let initial_balance = game.view().bank_balance;
        let initial_bet = game.view().player_hands[0].bet_amount;

        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.player_hands[0].bet_amount, initial_bet * 2);
        assert_eq!(view.bank_balance, initial_balance - initial_bet);
    }

    #[test]
    fn double_draws_one_card_and_ends_round() {
        let mut game = Blackjack::new();

        let initial_cards = game.view().player_hands[0].cards.len();
        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.player_hands[0].cards.len(), initial_cards + 1);
        assert_eq!(view.phase, Phase::RoundOver);
    }

    #[test]
    fn double_is_ignored_after_hit() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Hit);
        let balance_after_hit = game.view().bank_balance;

        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.bank_balance, balance_after_hit);
    }

    #[test]
    fn bet_resets_after_new_round() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Double);
        assert!(game.view().player_hands[0].bet_amount > 10);

        game.apply(PlayerAction::NewRound);

        let view = game.view();
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn stay_resolves_the_round() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, Phase::RoundOver);
        assert_ne!(view.result, GameResult::Pending);
    }

    #[test]
    fn natural_blackjack_pays_three_to_two() {
        let mut game = Blackjack::new();

        let mut player_hand = Hand::new();
        player_hand.add(Card::new(Suit::SPADES, Value::ACE));
        player_hand.add(Card::new(Suit::HEARTS, Value::TEN));

        game.player_hands[0].hand = player_hand;

        let mut dealer_hand = Hand::new();
        dealer_hand.add(Card::new(Suit::CLUBS, Value::NINE));
        dealer_hand.add(Card::new(Suit::DIAMONDS, Value::SEVEN));

        game.dealer_hand = dealer_hand;

        game.resolve_blackjack_or_continue();

        let view = game.view();
        assert_eq!(view.result, GameResult::PlayerWin);
        assert_eq!(view.bank_balance, 1_000 - 10 + 25);
    }
}

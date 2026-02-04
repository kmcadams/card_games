use crate::{
    bank::bank::Bank,
    cards::{hand::Hand, Card},
    game::blackjack::{
        rules::{self, SplitContext},
        types::{BlackjackState, PlayerAction, PlayerHand, Shoe, Table},
        view::{BlackjackView, PlayerHandView, VisibleCard},
        GameResult,
    },
};

pub struct Blackjack {
    state: BlackjackState,
    shoe: Shoe,
    table: Table,

    bank: Bank,
    result: GameResult,
}
impl Blackjack {
    pub fn new() -> Self {
        Blackjack {
            state: BlackjackState::Dealing,
            shoe: Shoe::new_shuffled(),
            table: Table {
                player_hands: vec![PlayerHand::new(10)],
                dealer_hand: Hand::new(),
            },

            bank: Bank::new(1_000),

            result: GameResult::Pending,
        }
    }

    fn deal_initial_cards(&mut self) {
        {
            let hand = &mut self.table.player_hands[0];
            hand.hand.add(self.shoe.draw());
            hand.hand.add(self.shoe.draw());
        }

        self.table.dealer_hand.add(self.shoe.draw());
        self.table.dealer_hand.add(self.shoe.draw());

        self.resolve_blackjack_or_continue();
    }

    fn resolve_blackjack_or_continue(&mut self) {
        let player = &self.table.player_hands[0];

        let player_blackjack = rules::is_blackjack(&player.hand);
        let dealer_blackjack = rules::is_blackjack(&self.table.dealer_hand);

        match (player_blackjack, dealer_blackjack) {
            (true, true) => {
                // Push
                self.bank.deposit(player.bet.amount);
                self.result = GameResult::Push;
                self.state = BlackjackState::RoundOver;
            }

            (true, false) => {
                // Player blackjack wins 3:2
                let payout = player.bet.amount + (player.bet.amount * 3 / 2);
                self.bank.deposit(payout);
                self.result = GameResult::PlayerWin;
                self.state = BlackjackState::RoundOver;
            }

            (false, true) => {
                // Dealer blackjack
                self.result = GameResult::DealerWin;
                self.state = BlackjackState::RoundOver;
            }

            (false, false) => {
                // Normal play continues
                self.state = BlackjackState::PlayerTurn { hand_index: 0 };
            }
        }
    }

    fn current_hand_idx(&self) -> usize {
        match self.state {
            BlackjackState::PlayerTurn { hand_index } => hand_index,
            _ => 0,
        }
    }

    pub fn start_round(&mut self) {
        self.table.player_hands = vec![PlayerHand::new(10)];

        self.table.dealer_hand.clear_hand();
        self.result = GameResult::Pending;

        self.state = BlackjackState::Dealing;

        let bet = self.table.player_hands[0].bet.amount;

        if !self.bank.withdraw(bet) {
            //TODO: add state/view for "out of money"
            return;
        }

        self.deal_initial_cards();
    }

    pub fn needs_shuffle(&self) -> bool {
        self.shoe.remaining() < 15
    }

    pub fn shuffle_shoe(&mut self) {
        self.shoe = Shoe::new_shuffled(); //TODO expand shoe to handle multiple shuffles
    }

    pub fn apply(&mut self, action: PlayerAction) {
        let snapshot = self.state;
        match snapshot {
            BlackjackState::Dealing => self.handle_dealing(action),
            BlackjackState::PlayerTurn { hand_index } => {
                self.handle_player_turn(hand_index, action)
            }
            BlackjackState::DealerTurn => {
                unreachable!("Dealer turn should be handled automatically without player input")
            }
            BlackjackState::RoundOver => self.handle_round_over(action),
        }
    }

    fn handle_dealing(&mut self, _action: PlayerAction) {
        // TODO better transition?
    }

    fn advance_to_dealer(&mut self) {
        self.state = BlackjackState::DealerTurn;
        self.play_dealer();
    }

    fn advance_player_turn_or_dealer(&mut self, idx: usize) {
        let next = idx + 1;

        if let Some((next_idx, _)) = self
            .table
            .player_hands
            .iter()
            .enumerate()
            .skip(next)
            .find(|(_, h)| !h.is_complete)
        {
            self.state = BlackjackState::PlayerTurn {
                hand_index: next_idx,
            };
        } else {
            self.advance_to_dealer();
        }
    }

    fn handle_player_turn(&mut self, idx: usize, action: PlayerAction) {
        let hand = &mut self.table.player_hands[idx];

        match action {
            PlayerAction::Hit => {
                hand.hand.add(self.shoe.draw());

                if rules::is_bust(&hand.hand) {
                    hand.is_complete = true;
                    self.advance_player_turn_or_dealer(idx);
                }
            }

            PlayerAction::Stay => {
                hand.is_complete = true;
                self.advance_player_turn_or_dealer(idx);
            }

            PlayerAction::Double => {
                // allowed only on the current hand
                if !rules::can_double(&hand.hand) {
                    return;
                }

                let bet = hand.bet.amount;
                if !self.bank.withdraw(bet) {
                    return;
                }

                hand.bet.amount *= 2;
                hand.hand.add(self.shoe.draw());
                hand.is_complete = true;

                // After double, move on (either next hand or dealer)
                self.advance_to_dealer();
            }

            PlayerAction::Split => {
                // only allow a single split for now
                let split_context = self.split_context();
                {
                    let hand = &self.table.player_hands[idx];
                    if !rules::can_split(&hand.hand, split_context) {
                        return;
                    }
                    if self.bank.balance() < hand.bet.amount {
                        return;
                    }
                }

                let bet = self.table.player_hands[idx].bet.amount;
                if !self.bank.withdraw(bet) {
                    return;
                }

                // Extract the two original cards without holding a borrow over self.draw_card()
                let cards = self.table.player_hands[idx].hand.cards().to_vec();
                let (c0, c1) = (cards[0], cards[1]);

                let new1 = self.shoe.draw();
                let new2 = self.shoe.draw();

                {
                    let hand = &mut self.table.player_hands[idx];
                    hand.hand.clear_hand();
                    hand.hand.add(c0);
                    hand.hand.add(new1);
                }

                let mut split_hand = PlayerHand::new(bet);
                split_hand.hand.add(c1);
                split_hand.hand.add(new2);

                self.table.player_hands.push(split_hand);

                // Continue playing primary first
                self.state = BlackjackState::PlayerTurn { hand_index: 0 };
            }

            _ => {}
        }
    }

    fn play_dealer(&mut self) {
        while rules::dealer_should_hit(&self.table.dealer_hand) {
            self.table.dealer_hand.add(self.shoe.draw());
        }

        self.resolve_round();
    }

    fn resolve_round(&mut self) {
        self.state = BlackjackState::RoundOver;
        let dealer_score = rules::hand_score(&self.table.dealer_hand);
        let dealer_bust = rules::is_bust(&self.table.dealer_hand);

        // You currently store only one GameResult; we’ll set it to “best/worst”
        // for now so UI has something to show. You can upgrade later.
        let mut any_win = false;
        let mut any_push = false;
        let mut any_loss = false;

        for hand in &self.table.player_hands {
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

        self.state = BlackjackState::RoundOver;
    }
    fn handle_round_over(&mut self, action: PlayerAction) {
        if action == PlayerAction::NewRound {
            if self.needs_shuffle() {
                self.shuffle_shoe();
            }
            self.start_round();
        }
    }

    fn split_context(&self) -> SplitContext {
        if self.table.player_hands.len() > 1 {
            SplitContext::AlreadySplit
        } else {
            SplitContext::NoPreviousSplit
        }
    }

    fn available_actions(&self) -> Vec<PlayerAction> {
        let mut controls = vec![PlayerAction::Quit];

        match self.state {
            BlackjackState::PlayerTurn { .. } => {
                let idx = self.current_hand_idx();
                let hand = &self.table.player_hands[idx];

                controls.insert(0, PlayerAction::Stay);
                controls.insert(0, PlayerAction::Hit);

                if rules::can_double(&hand.hand) && self.bank.balance() >= hand.bet.amount {
                    controls.insert(0, PlayerAction::Double);
                }

                if rules::can_split(&hand.hand, self.split_context())
                    && self.bank.balance() >= hand.bet.amount
                {
                    controls.insert(0, PlayerAction::Split);
                }
            }

            BlackjackState::RoundOver => {
                controls.insert(0, PlayerAction::NewRound);
            }
            _ => {}
        }

        controls
    }

    pub fn view(&self) -> BlackjackView {
        let active_hand_index = self.current_hand_idx();

        // Player hands (all face-up)
        let player_hands = self
            .table
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
        let (dealer_cards, dealer_visible_score, dealer_has_hidden_card) = match self.state {
            BlackjackState::Dealing | BlackjackState::PlayerTurn { .. } => {
                let visible_cards: Vec<Card> = self
                    .table
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
                    .table
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

            BlackjackState::DealerTurn | BlackjackState::RoundOver => {
                let cards: Vec<VisibleCard> = self
                    .table
                    .dealer_hand
                    .iter()
                    .cloned()
                    .map(VisibleCard::FaceUp)
                    .collect();

                let score = Some(rules::hand_score(&self.table.dealer_hand));

                (cards, score, false)
            }
        };

        let controls = self.available_actions();
        let total_bet: u32 = self.table.player_hands.iter().map(|h| h.bet.amount).sum();

        BlackjackView {
            available_actions: controls,
            phase: self.state,

            player_hands,
            active_hand_index,

            dealer_cards,
            dealer_visible_score,
            dealer_has_hidden_card,

            bank_balance: self.bank.balance(),
            total_bet,

            result: self.result,

            can_hit: matches!(self.state, BlackjackState::PlayerTurn { .. }),
            can_stay: matches!(self.state, BlackjackState::PlayerTurn { .. }),
            can_start_new_round: self.state == BlackjackState::RoundOver,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::{Card, Suit, Value};

    use super::*;

    #[test]
    fn new_round_withdraws_initial_bet() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            // Player
            Card::new(Suit::SPADES, Value::TWO),
            Card::new(Suit::CLUBS, Value::THREE),
            // Dealer
            Card::new(Suit::HEARTS, Value::FOUR),
            Card::new(Suit::DIAMONDS, Value::FIVE),
        ]);
        game.start_round();
        let view = game.view();

        assert_eq!(view.bank_balance, 1_000 - 10);
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn new_round_deals_initial_cards() {
        let mut game = Blackjack::new();
        game.start_round();
        let view = game.view();

        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.dealer_cards.len(), 2);
    }

    #[test]
    fn hit_adds_one_card_to_player_hand() {
        let mut game = Blackjack::new();
        game.start_round();

        let initial_cards = game.view().player_hands[0].cards.len();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.player_hands[0].cards.len(), initial_cards + 1);
    }

    #[test]
    fn stay_advances_to_round_over() {
        let mut game = Blackjack::new();
        game.start_round();
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn double_is_available_on_first_player_turn() {
        let mut game = Blackjack::new();
        game.start_round();
        let view = game.view();

        assert!(view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_is_not_available_after_hit() {
        let mut game = Blackjack::new();
        game.start_round();

        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert!(!view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_doubles_bet_and_withdraws_balance() {
        let mut game = Blackjack::new();
        game.start_round();

        // Force a valid double state
        game.table.player_hands[0].hand = {
            let mut h = Hand::new();
            h.add(Card::new(Suit::SPADES, Value::FIVE));
            h.add(Card::new(Suit::HEARTS, Value::SIX));
            h
        };

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
        game.start_round();

        let initial_cards = game.view().player_hands[0].cards.len();
        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.player_hands[0].cards.len(), initial_cards + 1);
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn double_is_ignored_after_hit() {
        let mut game = Blackjack::new();
        game.start_round();

        game.apply(PlayerAction::Hit);
        let balance_after_hit = game.view().bank_balance;

        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.bank_balance, balance_after_hit);
    }

    #[test]
    fn bet_resets_after_new_round() {
        let mut game = Blackjack::new();
        game.start_round();

        // Force a valid double
        game.table.player_hands[0].hand = {
            let mut h = Hand::new();
            h.add(Card::new(Suit::SPADES, Value::FIVE));
            h.add(Card::new(Suit::HEARTS, Value::SIX));
            h
        };

        game.apply(PlayerAction::Double);
        assert!(game.view().player_hands[0].bet_amount > 10);

        game.apply(PlayerAction::Stay);
        assert_eq!(game.view().phase, BlackjackState::RoundOver);

        game.apply(PlayerAction::NewRound);

        let view = game.view();
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn natural_blackjack_pays_three_to_two() {
        let mut game = Blackjack::new();
        game.start_round();
        let mut player_hand = Hand::new();
        player_hand.add(Card::new(Suit::SPADES, Value::ACE));
        player_hand.add(Card::new(Suit::HEARTS, Value::TEN));

        game.table.player_hands[0].hand = player_hand;

        let mut dealer_hand = Hand::new();
        dealer_hand.add(Card::new(Suit::CLUBS, Value::NINE));
        dealer_hand.add(Card::new(Suit::DIAMONDS, Value::SEVEN));

        game.table.dealer_hand = dealer_hand;

        game.resolve_blackjack_or_continue();

        let view = game.view();
        assert_eq!(view.result, GameResult::PlayerWin);
        assert_eq!(view.bank_balance, 1_000 - 10 + 25);
    }
}

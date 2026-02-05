use crate::{
    bank::bank::Bank,
    cards::{hand::Hand, Card},
    game::blackjack::{
        rules::{self, SplitContext},
        types::{BlackjackEvent, BlackjackState, PlayerAction, PlayerHand, Shoe, Table},
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

    fn deal_initial_cards(&mut self, events: &mut Vec<BlackjackEvent>) {
        {
            let hand = &mut self.table.player_hands[0];
            hand.hand.add(self.shoe.draw());
            hand.hand.add(self.shoe.draw());
        }

        self.table.dealer_hand.add(self.shoe.draw());
        self.table.dealer_hand.add(self.shoe.draw());

        self.resolve_blackjack_or_continue(events);
    }

    fn resolve_blackjack_or_continue(&mut self, events: &mut Vec<BlackjackEvent>) {
        let player = &self.table.player_hands[0];

        let player_blackjack = rules::is_blackjack(&player.hand);
        let dealer_blackjack = rules::is_blackjack(&self.table.dealer_hand);

        match (player_blackjack, dealer_blackjack) {
            (true, true) => {
                // Push
                self.bank.deposit(player.bet.amount);
                self.set_result(GameResult::Push, events);
                self.transition_to(BlackjackState::RoundOver, events);
            }

            (true, false) => {
                // Player blackjack wins 3:2
                let payout = player.bet.amount + (player.bet.amount * 3 / 2);
                self.bank.deposit(payout);
                self.set_result(GameResult::PlayerWin, events);
                self.transition_to(BlackjackState::RoundOver, events);
            }

            (false, true) => {
                // Dealer blackjack
                self.set_result(GameResult::DealerWin, events);
                self.transition_to(BlackjackState::RoundOver, events);
            }

            (false, false) => {
                // Normal play continues
                self.transition_to(BlackjackState::PlayerTurn { hand_index: 0 }, events);
            }
        }
    }

    fn current_hand_idx(&self) -> usize {
        match self.state {
            BlackjackState::PlayerTurn { hand_index } => hand_index,
            _ => 0,
        }
    }

    pub fn start_round(&mut self) -> Vec<BlackjackEvent> {
        let mut events = vec![];
        self.table.player_hands = vec![PlayerHand::new(10)];

        self.table.dealer_hand.clear_hand();
        self.result = GameResult::Pending;

        self.transition_to(BlackjackState::Dealing, &mut events);

        let bet = self.table.player_hands[0].bet.amount;

        if !self.bank.withdraw(bet) {
            //TODO: add state/view for "out of money"
            return events;
        }

        self.advance_automatic_transitions(&mut events);
        events
    }

    pub fn needs_shuffle(&self) -> bool {
        self.shoe.remaining() < 15
    }

    pub fn shuffle_shoe(&mut self) {
        self.shoe = Shoe::new_shuffled(); //TODO expand shoe to handle multiple shuffles
    }

    pub fn apply(&mut self, action: PlayerAction) -> Vec<BlackjackEvent> {
        let mut events = vec![];
        let snapshot = self.state;
        let handled = match snapshot {
            BlackjackState::PlayerTurn { hand_index } => {
                self.handle_player_turn(hand_index, action, &mut events)
            }
            BlackjackState::Dealing | BlackjackState::DealerTurn | BlackjackState::RoundOver => {
                // These phases are engine-driven; player input is ignored.
                events.push(BlackjackEvent::ActionIgnored {
                    action,
                    state: snapshot,
                });
                return events;
            }
        };

        if !handled {
            events.push(BlackjackEvent::ActionIgnored {
                action,
                state: snapshot,
            });
            return events;
        }

        events.insert(0, BlackjackEvent::ActionApplied { action });
        self.advance_automatic_transitions(&mut events);
        events
    }

    pub fn request_new_round(&mut self) -> Vec<BlackjackEvent> {
        if self.state != BlackjackState::RoundOver {
            return vec![BlackjackEvent::RoundStartIgnored { state: self.state }];
        }

        if self.needs_shuffle() {
            self.shuffle_shoe();
        }

        let mut events = vec![BlackjackEvent::RoundStarted];
        events.extend(self.start_round());
        events
    }

    fn advance_automatic_transitions(&mut self, events: &mut Vec<BlackjackEvent>) {
        loop {
            match self.state {
                BlackjackState::Dealing => self.deal_initial_cards(events),
                BlackjackState::DealerTurn => self.play_dealer(events),
                BlackjackState::PlayerTurn { .. } | BlackjackState::RoundOver => break,
            }
        }
    }

    fn transition_to(&mut self, next: BlackjackState, events: &mut Vec<BlackjackEvent>) {
        let previous = self.state;
        self.state = next;

        if previous != next {
            events.push(BlackjackEvent::StateChanged {
                from: previous,
                to: next,
            });
        }
    }

    fn set_result(&mut self, result: GameResult, events: &mut Vec<BlackjackEvent>) {
        self.result = result;
        if result != GameResult::Pending {
            events.push(BlackjackEvent::RoundResolved { result });
        }
    }

    fn advance_to_dealer(&mut self, events: &mut Vec<BlackjackEvent>) {
        self.transition_to(BlackjackState::DealerTurn, events);
    }

    fn advance_player_turn_or_dealer(&mut self, idx: usize, events: &mut Vec<BlackjackEvent>) {
        let next = idx + 1;

        if let Some((next_idx, _)) = self
            .table
            .player_hands
            .iter()
            .enumerate()
            .skip(next)
            .find(|(_, h)| !h.is_complete)
        {
            self.transition_to(
                BlackjackState::PlayerTurn {
                    hand_index: next_idx,
                },
                events,
            );
        } else {
            self.advance_to_dealer(events);
        }
    }

    fn handle_player_turn(
        &mut self,
        idx: usize,
        action: PlayerAction,
        events: &mut Vec<BlackjackEvent>,
    ) -> bool {
        let hand = &mut self.table.player_hands[idx];

        match action {
            PlayerAction::Hit => {
                hand.hand.add(self.shoe.draw());

                if rules::is_bust(&hand.hand) {
                    hand.is_complete = true;
                    self.advance_player_turn_or_dealer(idx, events);
                }
                true
            }

            PlayerAction::Stay => {
                hand.is_complete = true;
                self.advance_player_turn_or_dealer(idx, events);
                true
            }

            PlayerAction::Double => {
                // allowed only on the current hand
                if !rules::can_double(&hand.hand) {
                    return false;
                }

                let bet = hand.bet.amount;
                if !self.bank.withdraw(bet) {
                    return false;
                }

                hand.bet.amount *= 2;
                hand.hand.add(self.shoe.draw());
                hand.is_complete = true;

                // After double, move on (either next hand or dealer)
                self.advance_player_turn_or_dealer(idx, events);
                true
            }

            PlayerAction::Split => {
                // only allow a single split for now
                let split_context = self.split_context();
                {
                    let hand = &self.table.player_hands[idx];
                    if !rules::can_split(&hand.hand, split_context) {
                        return false;
                    }
                    if self.bank.balance() < hand.bet.amount {
                        return false;
                    }
                }

                let bet = self.table.player_hands[idx].bet.amount;
                if !self.bank.withdraw(bet) {
                    return false;
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
                self.transition_to(BlackjackState::PlayerTurn { hand_index: 0 }, events);
                true
            }
        }
    }

    fn play_dealer(&mut self, events: &mut Vec<BlackjackEvent>) {
        while rules::dealer_should_hit(&self.table.dealer_hand) {
            self.table.dealer_hand.add(self.shoe.draw());
        }

        self.resolve_round(events);
    }

    fn resolve_round(&mut self, events: &mut Vec<BlackjackEvent>) {
        let dealer_score = rules::hand_score(&self.table.dealer_hand);
        let dealer_bust = rules::is_bust(&self.table.dealer_hand);

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
        let result = if any_win && !any_loss {
            GameResult::PlayerWin
        } else if any_loss && !any_win && !any_push {
            GameResult::DealerWin
        } else if any_push && !any_win && !any_loss {
            GameResult::Push
        } else if any_win || any_push || any_loss {
            // Mixed outcomes; pick something neutral-ish.
            GameResult::Push
        } else {
            GameResult::Pending
        };

        self.set_result(result, events);
        self.transition_to(BlackjackState::RoundOver, events);
    }
    fn split_context(&self) -> SplitContext {
        if self.table.player_hands.len() > 1 {
            SplitContext::AlreadySplit
        } else {
            SplitContext::NoPreviousSplit
        }
    }

    fn available_actions(&self) -> Vec<PlayerAction> {
        let mut controls = vec![];

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
    use crate::{
        bank::bank::Bank,
        cards::{Card, Suit, Value},
    };

    use super::*;

    #[test]
    fn new_round_withdraws_initial_bet() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TWO),
            Card::new(Suit::CLUBS, Value::THREE),
            Card::new(Suit::HEARTS, Value::FOUR),
            Card::new(Suit::DIAMONDS, Value::FIVE),
        ]);
        game.start_round();
        let view = game.view();

        assert_eq!(view.bank_balance, 1_000 - 10);
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn start_round_resolves_dealing_before_waiting_for_player() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);

        game.start_round();

        assert!(matches!(
            game.view().phase,
            BlackjackState::PlayerTurn { .. } | BlackjackState::RoundOver
        ));
    }

    #[test]
    fn apply_ignores_input_during_engine_driven_states() {
        let mut game = Blackjack::new();
        game.state = BlackjackState::DealerTurn;
        let dealer_turn_events = game.apply(PlayerAction::Hit);
        assert_eq!(game.state, BlackjackState::DealerTurn);
        assert_eq!(
            dealer_turn_events,
            vec![BlackjackEvent::ActionIgnored {
                action: PlayerAction::Hit,
                state: BlackjackState::DealerTurn,
            }]
        );

        game.state = BlackjackState::Dealing;
        let dealing_events = game.apply(PlayerAction::Stay);
        assert_eq!(game.state, BlackjackState::Dealing);
        assert_eq!(
            dealing_events,
            vec![BlackjackEvent::ActionIgnored {
                action: PlayerAction::Stay,
                state: BlackjackState::Dealing,
            }]
        );

        game.state = BlackjackState::RoundOver;
        let round_over_events = game.apply(PlayerAction::Stay);
        assert_eq!(game.state, BlackjackState::RoundOver);
        assert_eq!(
            round_over_events,
            vec![BlackjackEvent::ActionIgnored {
                action: PlayerAction::Stay,
                state: BlackjackState::RoundOver,
            }]
        );
    }

    #[test]
    fn apply_stay_emits_action_and_transition_events() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();

        let events = game.apply(PlayerAction::Stay);

        assert!(events.contains(&BlackjackEvent::ActionApplied {
            action: PlayerAction::Stay
        }));
        assert!(events.contains(&BlackjackEvent::StateChanged {
            from: BlackjackState::PlayerTurn { hand_index: 0 },
            to: BlackjackState::DealerTurn,
        }));
        assert!(events.contains(&BlackjackEvent::StateChanged {
            from: BlackjackState::DealerTurn,
            to: BlackjackState::RoundOver,
        }));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BlackjackEvent::RoundResolved { .. })),
            "Expected a RoundResolved event after dealer play"
        );
    }

    #[test]
    fn new_round_deals_initial_cards() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();
        let view = game.view();

        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.dealer_cards.len(), 2);
    }

    #[test]
    fn hit_adds_one_card_to_player_hand() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::TWO),
        ]);
        game.start_round();

        let initial_cards = game.view().player_hands[0].cards.len();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.player_hands[0].cards.len(), initial_cards + 1);
    }

    #[test]
    fn stay_advances_to_round_over() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn double_is_available_on_first_player_turn() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();
        let view = game.view();

        assert!(view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_is_not_available_after_hit() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::TWO),
        ]);
        game.start_round();

        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert!(!view.available_actions.contains(&PlayerAction::Double));
    }

    #[test]
    fn double_doubles_bet_and_withdraws_balance() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::NINE),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::KING),
        ]);
        game.start_round();

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
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::NINE),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::KING),
        ]);
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
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::TWO),
        ]);
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
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::NINE),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::KING),
        ]);
        game.start_round();

        game.apply(PlayerAction::Double);
        assert!(game.view().player_hands[0].bet_amount > 10);

        game.apply(PlayerAction::Stay);
        assert_eq!(game.view().phase, BlackjackState::RoundOver);

        let events = game.request_new_round();
        assert!(events.contains(&BlackjackEvent::RoundStarted));

        let view = game.view();
        assert_eq!(view.player_hands[0].bet_amount, 10);
    }

    #[test]
    fn request_new_round_is_ignored_when_round_is_not_over() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),
            Card::new(Suit::HEARTS, Value::SIX),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();

        let events = game.request_new_round();

        assert_eq!(
            events,
            vec![BlackjackEvent::RoundStartIgnored {
                state: BlackjackState::PlayerTurn { hand_index: 0 },
            }]
        );
    }

    #[test]
    fn natural_blackjack_pays_three_to_two() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::ACE),
            Card::new(Suit::HEARTS, Value::TEN),
            Card::new(Suit::CLUBS, Value::NINE),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();
        let view = game.view();
        assert_eq!(view.result, GameResult::PlayerWin);
        assert_eq!(view.bank_balance, 1_000 - 10 + 25);
    }

    #[test]
    fn player_and_dealer_blackjack_is_push() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::ACE),
            Card::new(Suit::HEARTS, Value::TEN),
            Card::new(Suit::CLUBS, Value::ACE),
            Card::new(Suit::DIAMONDS, Value::KING),
        ]);

        game.start_round();

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::Push);
        assert_eq!(view.bank_balance, 1_000);
    }

    #[test]
    fn dealer_blackjack_wins_immediately() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TEN),
            Card::new(Suit::HEARTS, Value::NINE),
            Card::new(Suit::CLUBS, Value::ACE),
            Card::new(Suit::DIAMONDS, Value::KING),
        ]);

        game.start_round();

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::DealerWin);
        assert_eq!(view.bank_balance, 1_000 - 10);
    }

    #[test]
    fn split_creates_two_hands_and_withdraws_second_bet() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::TWO),
            Card::new(Suit::HEARTS, Value::THREE),
        ]);
        game.start_round();

        assert!(game.view().available_actions.contains(&PlayerAction::Split));
        game.apply(PlayerAction::Split);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::PlayerTurn { hand_index: 0 });
        assert_eq!(view.player_hands.len(), 2);
        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.player_hands[1].cards.len(), 2);
        assert_eq!(view.player_hands[0].bet_amount, 10);
        assert_eq!(view.player_hands[1].bet_amount, 10);
        assert_eq!(view.total_bet, 20);
        assert_eq!(view.bank_balance, 1_000 - 10 - 10);
    }

    #[test]
    fn split_is_unavailable_when_bank_cannot_cover_second_bet() {
        let mut game = Blackjack::new();
        game.bank = Bank::new(10);
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
        ]);
        game.start_round();

        let before = game.view().player_hands.len();
        assert!(!game.view().available_actions.contains(&PlayerAction::Split));
        game.apply(PlayerAction::Split);

        let view = game.view();
        assert_eq!(view.player_hands.len(), before);
        assert_eq!(view.bank_balance, 0);
    }

    #[test]
    fn double_after_split_advances_to_next_hand() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT),
            Card::new(Suit::HEARTS, Value::EIGHT),
            Card::new(Suit::CLUBS, Value::TEN),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::TWO),
            Card::new(Suit::HEARTS, Value::THREE),
            Card::new(Suit::CLUBS, Value::FOUR),
        ]);
        game.start_round();
        game.apply(PlayerAction::Split);
        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::PlayerTurn { hand_index: 1 });
        assert!(view.player_hands[0].is_complete);
        assert!(!view.player_hands[1].is_complete);
        assert_eq!(view.player_hands[0].bet_amount, 20);
        assert_eq!(view.bank_balance, 1_000 - 10 - 10 - 10);
    }

    #[test]
    fn mixed_split_outcome_does_not_leave_result_pending() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TEN),
            Card::new(Suit::HEARTS, Value::TEN),
            Card::new(Suit::CLUBS, Value::KING),
            Card::new(Suit::DIAMONDS, Value::QUEEN),
            Card::new(Suit::SPADES, Value::QUEEN),
            Card::new(Suit::HEARTS, Value::TWO),
        ]);
        game.start_round();
        game.apply(PlayerAction::Split);
        game.apply(PlayerAction::Stay);
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::Push);
    }
}

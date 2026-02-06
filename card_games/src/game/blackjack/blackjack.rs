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
        let hand = &mut self.table.player_hands[0];
        hand.hand.add(self.shoe.draw());
        self.table.dealer_hand.add(self.shoe.draw());

        hand.hand.add(self.shoe.draw());

        self.table.dealer_hand.add(self.shoe.draw());
    }

    fn resolve_blackjack_or_continue(&mut self) {
        let player = &self.table.player_hands[0];

        let player_blackjack = rules::is_blackjack(&player.hand.cards());
        let dealer_blackjack = rules::is_blackjack(&self.table.dealer_hand.cards());

        #[cfg(test)]
        {
            eprintln!(
                "RESOLVE BJ -> P: {:?} BJ: {}| D: {:?} BJ: {}",
                self.table.player_hands[0].hand.cards(),
                player_blackjack,
                self.table.dealer_hand.cards(),
                dealer_blackjack
            );
        }
        match (player_blackjack, dealer_blackjack) {
            (true, true) => {
                self.bank.deposit(self.table.player_hands[0].bet.amount);
                self.end_round(GameResult::Push);
            }
            (true, false) => {
                let bet = self.table.player_hands[0].bet.amount;
                self.bank.deposit(bet + bet * 3 / 2);
                self.end_round(GameResult::PlayerWin);
            }
            (false, true) => {
                self.end_round(GameResult::DealerWin);
            }
            (false, false) => {
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
        if self.state != BlackjackState::RoundOver && self.state != BlackjackState::Dealing {
            return;
        }

        self.table.player_hands = vec![PlayerHand::new(10)];
        self.table.dealer_hand.clear_hand();
        self.result = GameResult::Pending;

        self.state = BlackjackState::Dealing;

        let bet = self.table.player_hands[0].bet.amount;
        if !self.bank.withdraw(bet) {
            // TODO: out-of-money state
            return;
        }

        self.run_automatic();
        #[cfg(test)]
        {
            eprintln!(
                "START ROUND-> P: {:?} | D: {:?}",
                self.table.player_hands[0].hand.cards(),
                self.table.dealer_hand.cards()
            );
        }
    }

    fn end_round(&mut self, result: GameResult) {
        for hand in &mut self.table.player_hands {
            hand.is_complete = true;
        }

        self.result = result;
        self.state = BlackjackState::RoundOver;
        if self.needs_shuffle() {
            self.shuffle_shoe();
        }
    }

    pub fn needs_shuffle(&self) -> bool {
        self.shoe.remaining() < 15
    }

    pub fn shuffle_shoe(&mut self) {
        self.shoe = Shoe::new_shuffled(); //TODO expand shoe to handle multiple shuffles
    }

    pub fn apply(&mut self, action: PlayerAction) {
        let BlackjackState::PlayerTurn { hand_index } = self.state else {
            return;
        };

        self.apply_to_player_hand(hand_index, action);

        if self.table.player_hands[hand_index].is_complete {
            self.advance_after_hand_complete(hand_index);
        }

        self.run_automatic();
    }

    // pub fn apply(&mut self, action: PlayerAction) {
    //     let snapshot = self.state;
    //     if let BlackjackState::PlayerTurn { hand_index } = snapshot {
    //         self.handle_player_turn(hand_index, action);

    //         if self.table.player_hands[hand_index].is_complete {
    //             self.advance_after_hand_complete(hand_index);
    //         }

    //         self.advance_automatic_transitions();
    //     }
    // }

    fn run_automatic(&mut self) {
        loop {
            match self.state {
                BlackjackState::Dealing => {
                    self.deal_initial_cards();
                    self.resolve_blackjack_or_continue();
                }
                BlackjackState::DealerTurn => self.play_dealer(),
                BlackjackState::PlayerTurn { .. } | BlackjackState::RoundOver => break,
            }
        }
    }

    fn advance_after_hand_complete(&mut self, completed_idx: usize) {
        debug_assert!(self.table.player_hands[completed_idx].is_complete);

        let next = completed_idx + 1;

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
            self.state = BlackjackState::DealerTurn;
        }
    }

    fn apply_to_player_hand(&mut self, idx: usize, action: PlayerAction) {
        let hand = &mut self.table.player_hands[idx];

        match action {
            PlayerAction::Hit => {
                hand.hand.add(self.shoe.draw());

                if rules::is_bust(&hand.hand.cards()) {
                    hand.is_complete = true;
                }
            }

            PlayerAction::Stay => {
                hand.is_complete = true;
            }

            PlayerAction::Double => {
                // allowed only on the current hand
                if !rules::can_double(&hand.hand.cards()) {
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
                // self.advance_after_hand_complete(idx);
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
                self.state = BlackjackState::PlayerTurn { hand_index: idx };
            }
        }
    }

    fn play_dealer(&mut self) {
        while rules::dealer_should_hit(&self.table.dealer_hand.cards()) {
            self.table.dealer_hand.add(self.shoe.draw());
        }

        self.resolve_round();
    }

    fn resolve_round(&mut self) {
        let dealer_score = rules::hand_score(&self.table.dealer_hand.cards());
        let dealer_bust = rules::is_bust(&self.table.dealer_hand.cards());

        let mut any_win = false;
        let mut any_push = false;
        let mut any_loss = false;

        for hand in &self.table.player_hands {
            let player_score = rules::hand_score(&hand.hand.cards());
            let player_bust = rules::is_bust(&hand.hand.cards());

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

        self.end_round(result);
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
    use super::*;
    use crate::bank::bank::Bank;
    use crate::cards::{Card, Suit, Value};

    #[test]
    fn start_round_withdraws_initial_bet() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TWO),    // p1
            Card::new(Suit::CLUBS, Value::THREE),   // d hole
            Card::new(Suit::HEARTS, Value::FOUR),   // p2
            Card::new(Suit::DIAMONDS, Value::FIVE), // d up
        ]);

        game.start_round();

        assert_eq!(game.view().bank_balance, 1_000 - 10);
        assert_eq!(game.view().player_hands[0].bet_amount, 10);
    }

    #[test]
    fn initial_deal_is_two_cards_each() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),    // p1
            Card::new(Suit::HEARTS, Value::SIX),     // d hole
            Card::new(Suit::CLUBS, Value::TEN),      // p2
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up
        ]);

        game.start_round();
        let view = game.view();

        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.dealer_cards.len(), 2);
    }

    #[test]
    fn natural_blackjack_pays_three_to_two() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::ACE),     // p1
            Card::new(Suit::CLUBS, Value::NINE),     // d hole
            Card::new(Suit::HEARTS, Value::TEN),     // p2 -> blackjack
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up
        ]);

        game.start_round();
        let view = game.view();

        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::PlayerWin);
        assert_eq!(view.bank_balance, 1_000 - 10 + 25);
    }

    #[test]
    fn dealer_blackjack_wins_immediately() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::CLUBS, Value::NINE),   // p1
            Card::new(Suit::SPADES, Value::ACE),   // d hole
            Card::new(Suit::HEARTS, Value::SEVEN), // p2
            Card::new(Suit::DIAMONDS, Value::TEN), // d up -> blackjack
        ]);

        game.start_round();
        let view = game.view();

        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::DealerWin);
        assert_eq!(view.bank_balance, 1_000 - 10);
    }

    #[test]
    fn player_and_dealer_blackjack_is_push() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::ACE),    // p1
            Card::new(Suit::CLUBS, Value::ACE),     // d hole
            Card::new(Suit::HEARTS, Value::TEN),    // p2
            Card::new(Suit::DIAMONDS, Value::KING), // d up
        ]);

        game.start_round();
        let view = game.view();

        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::Push);
        assert_eq!(view.bank_balance, 1_000);
    }

    #[test]
    fn hit_adds_one_card() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::FIVE),    // p1
            Card::new(Suit::HEARTS, Value::SIX),     // d hole
            Card::new(Suit::CLUBS, Value::TEN),      // p2
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up
            Card::new(Suit::SPADES, Value::TWO),     // hit
        ]);

        game.start_round();
        let before = game.view().player_hands[0].cards.len();

        game.apply(PlayerAction::Hit);

        assert_eq!(game.view().player_hands[0].cards.len(), before + 1);
    }

    #[test]
    fn stay_runs_dealer_and_ends_round() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::HEARTS, Value::TEN),     // p1
            Card::new(Suit::SPADES, Value::FIVE),    // d hole
            Card::new(Suit::HEARTS, Value::SIX),     // p2 -> 16
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up -> 12
            Card::new(Suit::CLUBS, Value::TEN),      // dealer hits -> bust
        ]);

        game.start_round();
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
        assert_eq!(view.result, GameResult::PlayerWin);
    }

    #[test]
    fn double_doubles_bet_withdraws_and_ends_round() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::SIX),     // p1
            Card::new(Suit::HEARTS, Value::SIX),     // d hole
            Card::new(Suit::CLUBS, Value::SIX),      // p2 -> 11
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up
            Card::new(Suit::SPADES, Value::TEN),     // double draw
            Card::new(Suit::CLUBS, Value::TEN),      // dealer follow-up
        ]);

        game.start_round();
        let bal0 = game.view().bank_balance;

        game.apply(PlayerAction::Double);
        let view = game.view();

        assert_eq!(view.player_hands[0].bet_amount, 20);
        assert_eq!(view.bank_balance, bal0 - 10);
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn split_creates_two_hands_and_withdraws_second_bet() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT), // p1
            Card::new(Suit::HEARTS, Value::FIVE),  // d hole
            Card::new(Suit::CLUBS, Value::EIGHT),  // p2 -> pair
            Card::new(Suit::DIAMONDS, Value::SIX), // d up
            Card::new(Suit::CLUBS, Value::TWO),    // primary draw
            Card::new(Suit::HEARTS, Value::THREE), // split draw
        ]);

        game.start_round();
        assert!(game.view().available_actions.contains(&PlayerAction::Split));

        game.apply(PlayerAction::Split);
        let view = game.view();

        assert_eq!(view.player_hands.len(), 2);
        assert_eq!(view.total_bet, 20);
        assert_eq!(view.bank_balance, 1_000 - 20);
        assert_eq!(view.phase, BlackjackState::PlayerTurn { hand_index: 0 });
        assert_eq!(view.player_hands[0].cards.len(), 2);
        assert_eq!(view.player_hands[1].cards.len(), 2);
    }

    #[test]
    fn split_unavailable_if_bank_cannot_cover_second_bet() {
        let mut game = Blackjack::new();
        game.bank = Bank::new(10);
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT), // p1
            Card::new(Suit::HEARTS, Value::FIVE),  // d hole
            Card::new(Suit::CLUBS, Value::EIGHT),  // p2
            Card::new(Suit::DIAMONDS, Value::SIX), // d up
        ]);

        game.start_round();
        assert!(!game.view().available_actions.contains(&PlayerAction::Split));
    }

    #[test]
    fn double_after_split_advances_to_next_hand() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT), // p1
            Card::new(Suit::HEARTS, Value::FIVE),  // d hole
            Card::new(Suit::CLUBS, Value::EIGHT),  // p2
            Card::new(Suit::DIAMONDS, Value::SIX), // d up
            Card::new(Suit::CLUBS, Value::TWO),    // primary draw
            Card::new(Suit::HEARTS, Value::THREE), // split draw
            Card::new(Suit::SPADES, Value::TEN),   // double draw for primary
        ]);

        game.start_round();
        game.apply(PlayerAction::Split);
        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::PlayerTurn { hand_index: 1 });
        assert!(view.player_hands[0].is_complete);
        assert!(!view.player_hands[1].is_complete);
        assert_eq!(view.player_hands[0].bet_amount, 20);
        assert_eq!(view.bank_balance, 1_000 - 30);
    }

    #[test]
    fn input_is_ignored_when_not_player_turn() {
        let mut game = Blackjack::new();
        game.state = BlackjackState::DealerTurn;

        let snapshot = game.view();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.phase, snapshot.phase);
        assert_eq!(view.bank_balance, snapshot.bank_balance);
    }

    #[test]
    fn bust_ends_hand_and_prevents_further_actions() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TEN),
            Card::new(Suit::HEARTS, Value::TEN),
            Card::new(Suit::CLUBS, Value::NINE),
            Card::new(Suit::DIAMONDS, Value::SEVEN),
            Card::new(Suit::SPADES, Value::FIVE), // bust
        ]);

        game.start_round();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert!(view.player_hands[0].is_complete);
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn dealer_stands_on_soft_17() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TEN),   // p1
            Card::new(Suit::HEARTS, Value::ACE),   // d hole
            Card::new(Suit::CLUBS, Value::SEVEN),  // p2
            Card::new(Suit::DIAMONDS, Value::SIX), // d up -> soft 17
        ]);

        game.start_round();
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::RoundOver);
    }

    #[test]
    fn dealer_hits_on_16() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::TEN),
            Card::new(Suit::HEARTS, Value::TEN),
            Card::new(Suit::CLUBS, Value::SIX),
            Card::new(Suit::DIAMONDS, Value::SIX),
            Card::new(Suit::SPADES, Value::TEN), // dealer busts
        ]);

        game.start_round();
        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.result, GameResult::PlayerWin);
    }

    #[test]
    fn split_one_hand_bust_other_continues() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT),   // p1
            Card::new(Suit::HEARTS, Value::TEN),     // d hole
            Card::new(Suit::CLUBS, Value::EIGHT),    // p2
            Card::new(Suit::DIAMONDS, Value::SEVEN), // d up
            Card::new(Suit::SPADES, Value::TEN),     // primary split draw
            Card::new(Suit::CLUBS, Value::THREE),    // split hand draw
            Card::new(Suit::HEARTS, Value::TEN),     // primary hit -> bust
        ]);

        game.start_round();
        game.apply(PlayerAction::Split);
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.phase, BlackjackState::PlayerTurn { hand_index: 1 });
    }

    #[test]
    fn cannot_split_twice() {
        let mut game = Blackjack::new();
        game.shoe = Shoe::rigged(vec![
            Card::new(Suit::SPADES, Value::EIGHT),
            Card::new(Suit::HEARTS, Value::FIVE),
            Card::new(Suit::CLUBS, Value::EIGHT),
            Card::new(Suit::DIAMONDS, Value::SIX),
            Card::new(Suit::CLUBS, Value::EIGHT),
            Card::new(Suit::HEARTS, Value::EIGHT),
        ]);

        game.start_round();
        game.apply(PlayerAction::Split);

        let view = game.view();
        assert!(!view.available_actions.contains(&PlayerAction::Split));
    }
}

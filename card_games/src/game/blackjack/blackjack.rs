use crate::{
    bank::{bank::Bank, bet::Bet},
    cards::{deck_builder::DeckBuilder, hand::Hand, Card, Deck},
    game::blackjack::{
        rules,
        types::{Phase, PlayerAction},
        view::{BlackjackView, VisibleCard},
        GameResult,
    },
};

pub struct Blackjack {
    phase: Phase,
    deck: Deck,
    player_hand: Hand,
    dealer_hand: Hand,
    bank: Bank,
    bet: Bet,
    result: GameResult,
}
impl Blackjack {
    pub fn new() -> Self {
        let mut game = Blackjack {
            phase: Phase::Dealing,
            deck: DeckBuilder::new().standard52().build(),
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),

            bank: Bank::new(1_000),
            bet: Bet { amount: 10 },
            result: GameResult::Pending,
        };

        game.new_round();
        game
    }

    fn deal_initial_cards(&mut self) {
        let card = self.draw_card();
        self.player_hand.add(card);
        let card = self.draw_card();
        self.player_hand.add(card);
        let card = self.draw_card();

        self.dealer_hand.add(card);
        let card = self.draw_card();

        self.dealer_hand.add(card);

        self.phase = Phase::PlayerTurn;
    }

    pub fn new_round(&mut self) {
        self.player_hand.clear_hand();
        self.dealer_hand.clear_hand();
        self.result = GameResult::Pending;
        self.bet.amount = 10;
        self.phase = Phase::Dealing;

        self.deck = DeckBuilder::new().standard52().build();
        self.deck.shuffle();
        if !self.bank.withdraw(self.bet.amount) {
            //TODO: add state/view for "out of money"
            return;
        }

        self.deal_initial_cards();
    }
    pub fn apply(&mut self, action: PlayerAction) {
        match (self.phase, action) {
            (Phase::PlayerTurn, PlayerAction::Hit) => {
                let card = self.draw_card();
                self.player_hand.add(card);

                if rules::is_bust(&self.player_hand) {
                    self.phase = Phase::RoundOver;
                    self.result = GameResult::DealerWin;
                }
            }

            (Phase::PlayerTurn, PlayerAction::Stay) => {
                self.phase = Phase::DealerTurn;
                self.play_dealer();
            }

            (Phase::RoundOver, PlayerAction::NewRound) => {
                self.new_round();
            }

            (Phase::PlayerTurn, PlayerAction::Double)
                if rules::can_double(&self.player_hand) && self.bank.withdraw(self.bet.amount) =>
            {
                self.bet.amount *= 2;

                let card = self.draw_card();
                self.player_hand.add(card);

                self.phase = Phase::DealerTurn;
                self.play_dealer();
            }

            (Phase::PlayerTurn, PlayerAction::Split) => {
                todo!()
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
        let player_score = rules::hand_score(&self.player_hand);
        let dealer_score = rules::hand_score(&self.dealer_hand);

        self.result = GameResult::determine(player_score, dealer_score);

        match self.result {
            GameResult::PlayerWin => {
                self.bank.deposit(self.bet.amount * 2);
            }
            GameResult::Push => {
                self.bank.deposit(self.bet.amount);
            }
            GameResult::DealerWin => {
                // bet already lost
            }
            _ => {}
        }

        self.phase = Phase::RoundOver;
    }

    fn draw_card(&mut self) -> Card {
        self.deck
            .draw()
            .expect("Deck exhausted during Blackjack round") //TODO: remove expect
    }
    pub fn view(&self) -> BlackjackView {
        // Player cards are always fully visible
        let player_cards = self
            .player_hand
            .iter()
            .cloned()
            .map(VisibleCard::FaceUp)
            .collect();

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

        let mut controls = vec![PlayerAction::Quit];

        if self.phase == Phase::PlayerTurn {
            controls.insert(0, PlayerAction::Stay);
            controls.insert(0, PlayerAction::Hit);

            if rules::can_double(&self.player_hand) && self.bank.balance() >= self.bet.amount {
                controls.insert(0, PlayerAction::Double);
            }
        }

        if self.phase == Phase::RoundOver {
            controls.insert(0, PlayerAction::NewRound);
        }

        BlackjackView {
            available_actions: controls,
            phase: self.phase,
            player_cards,
            dealer_cards,
            player_score: rules::hand_score(&self.player_hand),
            dealer_visible_score,
            dealer_has_hidden_card,
            bet_amount: self.bet.amount,
            bank_balance: self.bank.balance(),
            result: self.result,
            can_hit: self.phase == Phase::PlayerTurn,
            can_stay: self.phase == Phase::PlayerTurn,
            can_start_new_round: self.phase == Phase::RoundOver,
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn new_round_withdraws_initial_bet() {
        let game = Blackjack::new();
        let view = game.view();

        assert_eq!(view.bank_balance, 1_000 - 10);
        assert_eq!(view.bet_amount, 10);
    }

    #[test]
    fn new_round_deals_initial_cards() {
        let game = Blackjack::new();
        let view = game.view();

        assert_eq!(view.player_cards.len(), 2);
        assert_eq!(view.dealer_cards.len(), 2);
    }

    #[test]
    fn hit_adds_one_card_to_player_hand() {
        let mut game = Blackjack::new();

        let initial_cards = game.view().player_cards.len();
        game.apply(PlayerAction::Hit);

        let view = game.view();
        assert_eq!(view.player_cards.len(), initial_cards + 1);
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
        let initial_bet = game.view().bet_amount;

        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.bet_amount, initial_bet * 2);
        assert_eq!(view.bank_balance, initial_balance - initial_bet);
    }

    #[test]
    fn double_draws_one_card_and_ends_round() {
        let mut game = Blackjack::new();

        let initial_cards = game.view().player_cards.len();
        game.apply(PlayerAction::Double);

        let view = game.view();
        assert_eq!(view.player_cards.len(), initial_cards + 1);
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
        assert!(game.view().bet_amount > 10);

        game.apply(PlayerAction::NewRound);

        let view = game.view();
        assert_eq!(view.bet_amount, 10);
    }

    #[test]
    fn stay_resolves_the_round() {
        let mut game = Blackjack::new();

        game.apply(PlayerAction::Stay);

        let view = game.view();
        assert_eq!(view.phase, Phase::RoundOver);
        assert_ne!(view.result, GameResult::Pending);
    }
}

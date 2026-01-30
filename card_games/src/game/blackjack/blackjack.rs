use crate::{
    cards::{card, deck_builder::DeckBuilder, hand::Hand, Card, Deck},
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
    result: GameResult,
}
impl Blackjack {
    pub fn new() -> Self {
        let mut game = Blackjack {
            phase: Phase::Dealing,
            deck: DeckBuilder::new().standard52().build(),
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),
            result: GameResult::Pending,
        };

        game.deck.shuffle();

        game.deal_initial_cards();
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
        self.phase = Phase::Dealing;

        self.deck = DeckBuilder::new().standard52().build();
        self.deck.shuffle();

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

            (Phase::PlayerTurn, PlayerAction::Double) => {
                todo!()
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

        self.result = match (player_score > 21, dealer_score > 21) {
            (true, _) => GameResult::DealerWin,
            (_, true) => GameResult::PlayerWin,
            _ if player_score > dealer_score => GameResult::PlayerWin,
            _ if dealer_score > player_score => GameResult::DealerWin,
            _ => GameResult::Push,
        };

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
            result: self.result,
            can_hit: self.phase == Phase::PlayerTurn,
            can_stay: self.phase == Phase::PlayerTurn,
            can_start_new_round: self.phase == Phase::RoundOver,
        }
    }
}

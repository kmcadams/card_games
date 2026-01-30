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

    pub fn apply(&mut self, action: PlayerAction) {
        if self.phase != Phase::PlayerTurn {
            return;
        }

        match action {
            PlayerAction::Hit => {
                let card = self.draw_card();
                self.player_hand.add(card);

                if rules::is_bust(&self.player_hand) {
                    self.phase = Phase::RoundOver;
                    self.result = GameResult::DealerWin;
                }
            }

            PlayerAction::Stay => {
                self.phase = Phase::DealerTurn;
                self.play_dealer();
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
        let player_cards = self
            .player_hand
            .iter()
            .cloned()
            .map(VisibleCard::FaceUp)
            .collect();

        let dealer_cards = match self.phase {
            Phase::Dealing | Phase::PlayerTurn => self
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
                .collect(),

            Phase::DealerTurn | Phase::RoundOver => self
                .dealer_hand
                .iter()
                .cloned()
                .map(VisibleCard::FaceUp)
                .collect(),
        };

        BlackjackView {
            phase: self.phase,
            player_cards,
            dealer_cards,
            player_score: rules::hand_score(&self.player_hand),
            dealer_score: if self.phase == Phase::RoundOver {
                Some(rules::hand_score(&self.dealer_hand))
            } else {
                None
            },
            result: self.result,
            can_hit: self.phase == Phase::PlayerTurn,
            can_stay: self.phase == Phase::PlayerTurn,
        }
    }
}

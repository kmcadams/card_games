use crate::{
    cards::{deck_builder::DeckBuilder, hand::Hand, Card, Deck},
    game::blackjack::{
        rules,
        types::{Phase, PlayerAction},
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
        Blackjack {
            phase: Phase::Dealing,
            deck: DeckBuilder::new().standard52().build(),
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),
            result: GameResult::Pending,
        }
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
    //TODO: Build a view
    //pub fn view(&self) -> BlackjackView;
}

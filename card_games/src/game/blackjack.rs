use std::cmp::Ordering;

use super::game::Game;
use super::input::PlayerInput;
use crate::cards::deck_builder::DeckBuilder;
use crate::cards::Deck;
use crate::player::Player;

use crate::game::rules::BlackjackRules;
use crate::ui::blackjack_display::BlackjackDisplay;

pub struct BlackjackPlayers {
    pub player: Player,
    pub dealer: Player,
}

pub enum Turn {
    Player,
    Dealer,
    Done,
}

enum GameState {
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameResult {
    Pending,
    PlayerWin,
    DealerWin,
    Push,
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Pending => write!(f, "⏳ Game in progress..."),
            GameResult::PlayerWin => write!(f, "🎉 You win!"),
            GameResult::DealerWin => write!(f, "💥 Dealer wins!"),
            GameResult::Push => write!(f, "🤝 Push!"),
        }
    }
}

pub struct BlackjackGame<I: PlayerInput, D: BlackjackDisplay> {
    input: I,
    display: D,
    pub deck: Deck,
    players: BlackjackPlayers,
    turn: Turn,
    state: GameState,
    pub result: GameResult,
}

impl<I, D> BlackjackGame<I, D>
where
    I: PlayerInput,
    D: BlackjackDisplay,
{
    pub fn new(input: I, display: D) -> Self {
        let mut deck = DeckBuilder::new().standard52().build();
        deck.shuffle();

        let player = Player::new("You".into());
        let dealer = Player::default();

        Self {
            input,
            display,
            deck,
            players: BlackjackPlayers { player, dealer },
            turn: Turn::Player,
            state: GameState::InProgress,
            result: GameResult::Pending,
        }
    }

    fn handle_player_turn(&mut self) {
        let player = &mut self.players.player;
        let dealer = &self.players.dealer;

        let visible_card = dealer.hand.cards().first().unwrap();

        self.display.show_turn(&self.turn);
        self.display
            .show_message(&format!("Dealer is showing: |{}|", visible_card));
        self.display.show_hand("You", &player.hand);

        let score = BlackjackRules::hand_score(player.hand.cards());
        self.display.show_score("You", score);

        if BlackjackRules::is_bust(player.hand.cards()) {
            self.display.show_message("You bust!");
            self.result = GameResult::DealerWin;
            self.turn = Turn::Done;
            return;
        }

        let choice = self.input.choose_action(&player.hand);

        match choice.as_str() {
            "h" | "hit" => {
                if let Some(card) = self.deck.draw() {
                    self.display.show_card_drawn(&card);
                    player.hand.add(card);

                    // Recalculate score after hit
                    let score = BlackjackRules::hand_score(player.hand.cards());
                    self.display.show_score("You", score);

                    if BlackjackRules::is_bust(player.hand.cards()) {
                        self.display.show_message("You bust!");
                        self.result = GameResult::DealerWin;
                        self.turn = Turn::Done;
                    }
                }
            }
            "s" | "stay" => {
                self.display.show_message("You chose to stay.");
                self.turn = Turn::Dealer;
            }
            _ => {
                self.display
                    .show_message("Invalid input. Please type 'h' or 's'.");
            }
        }
    }

    fn handle_dealer_turn(&mut self) {
        let dealer = &mut self.players.dealer;

        self.display.show_turn(&self.turn);
        self.display.show_hand("Dealer", &dealer.hand);

        let mut score = BlackjackRules::hand_score(dealer.hand.cards());
        self.display.show_score("Dealer", score);

        while score < 17 {
            self.display.show_message("Dealer hits.");
            if let Some(card) = self.deck.draw() {
                self.display.show_card_drawn(&card);
                dealer.hand.add(card);
                score = BlackjackRules::hand_score(dealer.hand.cards());
                self.display.show_score("Dealer", score);
            } else {
                self.display
                    .show_message("Deck is empty. Dealer cannot draw.");
                break;
            }
        }

        if score >= 17 && score <= 21 {
            self.display.show_message("Dealer stays.");
        } else if score > 21 {
            self.display.show_message("Dealer busted!");
        }

        self.display.show_hand("Dealer", &dealer.hand);
        self.display.show_score("Dealer", score);
        self.turn = Turn::Done;
    }

    fn end_game(&mut self) {
        let player = &self.players.player;
        let dealer = &self.players.dealer;

        let p_score = BlackjackRules::hand_score(player.hand.cards());
        let d_score = BlackjackRules::hand_score(dealer.hand.cards());

        self.display.show_message(&format!(
            "Your final hand: {} (Score: {})",
            player.hand, p_score
        ));

        self.display.show_message(&format!(
            "Dealer final hand: {} (Score: {})",
            dealer.hand, d_score
        ));
        let result = determine_result(p_score, d_score);

        self.display.show_result(&result);
        self.result = result;
        self.state = GameState::Finished
    }
}

impl<I, D> Game for BlackjackGame<I, D>
where
    I: PlayerInput,
    D: BlackjackDisplay,
{
    type Outcome = GameResult;
    fn setup(&mut self) {
        let _ = self
            .deck
            .deal(2, [&mut self.players.player, &mut self.players.dealer])
            .unwrap();
    }

    fn play(&mut self) {
        while !self.is_finished() {
            match self.turn {
                Turn::Player => self.handle_player_turn(),
                Turn::Dealer => self.handle_dealer_turn(),
                Turn::Done => {
                    self.end_game();
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, GameState::Finished)
    }

    fn winner(&self) -> GameResult {
        self.result
    }
}

fn determine_result(p_score: u8, d_score: u8) -> GameResult {
    if p_score > 21 {
        GameResult::DealerWin
    } else if d_score > 21 {
        GameResult::PlayerWin
    } else {
        match p_score.cmp(&d_score) {
            Ordering::Greater => GameResult::PlayerWin,
            Ordering::Less => GameResult::DealerWin,
            Ordering::Equal => GameResult::Push,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_result_returns_dealer_win_if_player_busts() {
        let result = determine_result(22, 18);
        assert_eq!(result, GameResult::DealerWin);
    }
}

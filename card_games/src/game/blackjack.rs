use std::cmp::Ordering;

use super::game::Game;
use super::input::PlayerInput;
use crate::cards::deck_builder::DeckBuilder;
use crate::cards::Deck;
use crate::player::Player;

use crate::game::rules::BlackjackRules;

pub struct BlackjackPlayers {
    pub player: Player,
    pub dealer: Player,
}

pub enum Turn {
    Player,
    Dealer,
    Done,
}

#[derive(Debug, PartialEq)]
pub enum GameResult {
    PlayerWin,
    DealerWin,
    Push,
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::PlayerWin => write!(f, "🎉 You win!"),
            GameResult::DealerWin => write!(f, "💥 Dealer wins!"),
            GameResult::Push => write!(f, "🤝 Push!"),
        }
    }
}

pub struct BlackjackGame<I: PlayerInput> {
    input: I,
    pub deck: Deck,
    pub players: BlackjackPlayers,
    pub turn: Turn,
}

impl<I: PlayerInput> BlackjackGame<I> {
    pub fn new(input: I) -> Self {
        let mut deck = DeckBuilder::new().standard52().build();
        deck.shuffle();

        let player = Player::new("You".into());
        let dealer = Player::default();

        Self {
            input,
            deck,
            players: BlackjackPlayers { player, dealer },
            turn: Turn::Player,
        }
    }

    fn handle_player_turn(&mut self) -> Option<GameResult> {
        let player = &mut self.players.player;
        let dealer = &self.players.dealer;

        let visible_card = dealer.hand.cards().first().unwrap();

        println!("\n=== Your Turn ===");
        println!("Dealer is showing: |{}|", visible_card);
        println!("Your hand: {}", player.hand);

        let score = BlackjackRules::hand_score(player.hand.cards());
        println!("Your current score: {}\n", score);

        if BlackjackRules::is_bust(player.hand.cards()) {
            println!("You busted!");
            self.turn = Turn::Done;
            return Some(GameResult::DealerWin);
        }

        let choice = self.input.choose_action(&player.hand);

        match choice.as_str() {
            "h" | "hit" => {
                if let Some(card) = self.deck.draw() {
                    println!("You drew: {}", card);
                    player.hand.add(card);
                }
            }
            "s" | "stay" => {
                println!("You chose to stay.");
                self.turn = Turn::Dealer;
            }
            _ => {
                println!("Invalid input. Please type 'h' or 's'.");
            }
        }
        None
    }

    fn handle_dealer_turn(&mut self) {
        let dealer = &mut self.players.dealer;

        println!("\n=== Dealer's Turn ===");
        println!("Dealer's hand: {}", dealer.hand);

        let mut score = BlackjackRules::hand_score(dealer.hand.cards());
        println!("Dealer score: {}", score);

        while score < 17 {
            println!("Dealer hits.");
            if let Some(card) = self.deck.draw() {
                println!("Dealer draws: {}", card);
                dealer.hand.add(card);
                score = BlackjackRules::hand_score(dealer.hand.cards());
                println!("Updated dealer score: {}", score);
            } else {
                println!("Deck is empty. Dealer cannot draw.");
                break;
            }
        }

        if score >= 17 && score <= 21 {
            println!("Dealer stays.");
        } else if score > 21 {
            println!("Dealer busted!");
        }

        println!("Dealer's final hand: {}", dealer.hand);
        println!("Dealer final score: {}", score);
        self.turn = Turn::Done;
    }

    fn end_game(&self) -> GameResult {
        let player = &self.players.player;
        let dealer = &self.players.dealer;

        let p_score = BlackjackRules::hand_score(player.hand.cards());
        let d_score = BlackjackRules::hand_score(dealer.hand.cards());

        println!("Your final hand: {} (Score: {})", player.hand, p_score);
        println!("Dealer final hand: {} (Score: {})", dealer.hand, d_score);

        determine_result(p_score, d_score)
    }
}

impl<I: PlayerInput> Game for BlackjackGame<I> {
    type Outcome = GameResult;
    fn setup(&mut self) {
        let _ = self
            .deck
            .deal(2, [&mut self.players.player, &mut self.players.dealer])
            .unwrap();
    }

    fn play_round(&mut self) -> Option<GameResult> {
        match self.turn {
            Turn::Player => self.handle_player_turn(),
            Turn::Dealer => {
                self.handle_dealer_turn();
                None
            }
            Turn::Done => Some(self.end_game()),
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.turn, Turn::Done)
    }

    fn winner(&self) -> Option<String> {
        Some("Unimplemented".to_string())
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

use std::collections::HashMap;

use card_games::{
    cards::{deck_type::DeckType, Deck},
    game::{BlackjackGame, Game},
    player::Player,
};

fn main() {
    println!("Welcome to Card Games");

    let mut game = BlackjackGame::new();
    game.setup();

    loop {
        if let Some(result) = game.play_round() {
            println!("\n=== Game Over ===");
            println!("{}", result);
            break;
        }
    }

    // let mut deck = Deck::new(DeckType::Standard52);
    // print!(
    //     "Your deck: {}\nNum of Cards: {}\n",
    //     deck,
    //     deck.remaining_cards()
    // );

    // deck.shuffle();
    // println!("\n\nShuffled deck:\n{}", deck);

    // let mut players: HashMap<u8, Player> = HashMap::new();
    // players.insert(2, Player::default());
    // players.insert(1, Player::new("Kyle".to_string()));

    // if let Err(err) = deck.deal(2, &mut players) {
    //     println!("Error: {}", err);
    // }
    // for (_, v) in &players {
    //     println!("{}'s Hand: {}", v.name(), v.hand);
    // }
    // println!(
    //     "Updated Deck:\n{}\nNum of Cards remaining in Deck: {}",
    //     deck,
    //     deck.remaining_cards()
    // );
}

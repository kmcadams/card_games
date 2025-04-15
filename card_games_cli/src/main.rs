use std::collections::HashMap;

use card_games::{cards::Deck, player::Player};

fn main() {
    println!("Welcome to Card Games");

    let mut deck = Deck::new();
    print!(
        "Your deck: {}\nNum of Cards: {}\n",
        deck,
        deck.remaining_cards()
    );

    deck.shuffle();
    println!("\n\nShuffled deck:\n{}", deck);

    let mut players: HashMap<u8, Player> = HashMap::new();
    players.insert(2, Player::default());
    players.insert(1, Player::new("Kyle".to_string()));

    deck.deal(2, &mut players);

    for (_, v) in &players {
        println!("{}'s Hand: {}", v.name(), v.hand);
    }
    println!(
        "Updated Deck:\n{}\nNum of Cards remaining in Deck: {}",
        deck,
        deck.remaining_cards()
    );
}

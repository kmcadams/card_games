mod cards;

use cards::deck::Deck;

fn main() {
    println!("Welcome to Card Games");

   let mut deck = Deck::new();
    print!("Your deck: {}\nNum of Cards: {}\n", deck, deck.remaining_cards());

    let shuffled_deck = deck.shuffle();
    println!("\n\nShuffled deck:\n{}", shuffled_deck);

}

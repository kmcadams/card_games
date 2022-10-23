mod cards;

use cards::deck::Deck;

fn main() {
    println!("Welcome to Card Games");

   let deck = Deck::new();
    print!("Your deck: {}\nNum of Cards: {}\n", deck, deck.remaining_cards());

}

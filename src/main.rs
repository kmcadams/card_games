mod cards;

use cards::deck::Deck;
use cards::hand::Hand;

fn main() {
    println!("Welcome to Card Games");

   let mut deck = Deck::new();
    print!("Your deck: {}\nNum of Cards: {}\n", deck, deck.remaining_cards());

    // let mut shuffled_deck = 
    deck.shuffle();
    println!("\n\nShuffled deck:\n{}", deck);

    let mut hand = Hand::new();
    deck.deal(2, &mut hand);
    println!("Player's hand: {}\nUpdated Deck:\n{}\nNum of Casrds remaining in Deck: {}",hand,deck, deck.remaining_cards());

}

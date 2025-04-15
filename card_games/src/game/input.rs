use crate::cards::hand::Hand;

pub trait PlayerInput {
    fn choose_action(&self, hand: &Hand) -> String;
}

pub struct TerminalInput;

impl PlayerInput for TerminalInput {
    fn choose_action(&self, _hand: &Hand) -> String {
        use std::io::{self, Write};
        print!("Do you want to [h]it or [s]tay? ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_lowercase()
    }
}

pub struct FixedInput {
    pub response: String,
}

impl PlayerInput for FixedInput {
    fn choose_action(&self, _hand: &Hand) -> String {
        self.response.clone()
    }
}

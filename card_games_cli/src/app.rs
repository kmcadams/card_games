use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use card_games::game::blackjack::{
    blackjack::{self, Blackjack},
    types::PlayerAction,
    view::BlackjackView,
};

pub struct App {
    game: Blackjack,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: Blackjack::new(),
            should_quit: false,
        }
    }
    pub fn view(&self) -> BlackjackView {
        self.game.view()
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), _) => {
                    self.should_quit = true;
                }
                (KeyCode::Char('h'), _) => {
                    self.game.apply(PlayerAction::Hit);
                }
                (KeyCode::Char('s'), _) => {
                    self.game.apply(PlayerAction::Stay);
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

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
        let action = match event {
            Event::Key(KeyEvent { code, .. }) => Self::map_key_to_action(code),
            _ => None,
        };

        match action {
            Some(PlayerAction::Quit) => {
                self.should_quit = true;
            }

            Some(PlayerAction::NewRound) => {
                self.game.apply(PlayerAction::NewRound);
            }

            Some(action) => {
                self.game.apply(action);
            }

            None => {}
        }
    }

    fn map_key_to_action(code: KeyCode) -> Option<PlayerAction> {
        match code {
            KeyCode::Char('h') => Some(PlayerAction::Hit),
            KeyCode::Char('s') => Some(PlayerAction::Stay),
            KeyCode::Char('d') => Some(PlayerAction::Double),
            KeyCode::Char('p') => Some(PlayerAction::Split),
            KeyCode::Char('n') => Some(PlayerAction::NewRound),
            KeyCode::Char('q') => Some(PlayerAction::Quit),
            _ => None,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

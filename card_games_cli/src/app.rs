use crossterm::event::{Event, KeyCode, KeyEvent};

use card_games::game::blackjack::{blackjack::Blackjack, types::PlayerAction, view::BlackjackView};

enum AppCommand {
    Action(PlayerAction),
    NewRound,
    Quit,
}

pub struct App {
    game: Blackjack,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut game = Blackjack::new();
        game.start_round();
        Self {
            game,
            should_quit: false,
        }
    }
    pub fn view(&self) -> BlackjackView {
        self.game.view()
    }

    pub fn handle_event(&mut self, event: Event) {
        let command = match event {
            Event::Key(KeyEvent { code, .. }) => Self::map_key_to_command(code),
            _ => None,
        };

        match command {
            Some(AppCommand::Quit) => {
                self.should_quit = true;
            }

            Some(AppCommand::NewRound) => {
                self.game.start_round();
            }

            Some(AppCommand::Action(action)) => {
                self.game.apply(action);
            }

            None => {}
        }
    }

    fn map_key_to_command(code: KeyCode) -> Option<AppCommand> {
        match code {
            KeyCode::Char('h') => Some(AppCommand::Action(PlayerAction::Hit)),
            KeyCode::Char('s') => Some(AppCommand::Action(PlayerAction::Stay)),
            KeyCode::Char('d') => Some(AppCommand::Action(PlayerAction::Double)),
            KeyCode::Char('p') => Some(AppCommand::Action(PlayerAction::Split)),
            KeyCode::Char('n') => Some(AppCommand::NewRound),
            KeyCode::Char('q') => Some(AppCommand::Quit),
            _ => None,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

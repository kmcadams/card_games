use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

mod app;
mod ui;

use app::App;

fn main() -> anyhow::Result<()> {
    setup_terminal()?;
    let result = run_app();
    restore_terminal()?;
    result
}

fn setup_terminal() -> anyhow::Result<()> {
    enable_raw_mode()?;

    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    Ok(())
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;

    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

fn run_app() -> anyhow::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        let view = app.view();

        terminal.draw(|frame| {
            ui::draw(frame, &view);
        })?;

        if app.should_quit() {
            break;
        }

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            let event = crossterm::event::read()?;
            app.handle_event(event);
        }
    }

    Ok(())
}

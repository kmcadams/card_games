use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use card_games::game::blackjack::{
    types::{GameResult, PlayerAction},
    view::{BlackjackView, VisibleCard},
};

/// Entry point called from `terminal.draw(|f| ...)`
pub fn draw(f: &mut Frame, view: &BlackjackView) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // dealer
            Constraint::Length(7), // player
            Constraint::Min(3),    // status / controls
        ])
        .split(f.area());

    draw_dealer(f, chunks[0], view);
    draw_player(f, chunks[1], view);
    draw_status(f, chunks[2], view);
}

fn draw_dealer(f: &mut Frame, area: ratatui::layout::Rect, view: &BlackjackView) {
    let cards = render_cards(&view.dealer_cards);

    let score = match (view.dealer_visible_score, view.dealer_has_hidden_card) {
        (Some(score), true) => format!("Score: {} + ?", score),
        (Some(score), false) => format!("Score: {}", score),
        _ => "Score: ?".to_string(),
    };

    let text = vec![Line::from("Dealer"), Line::from(cards), Line::from(score)];

    let block = Block::default().title("Dealer").borders(Borders::ALL);

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_player(f: &mut Frame, area: ratatui::layout::Rect, view: &BlackjackView) {
    let cards = render_cards(&view.player_cards);

    let text = vec![
        Line::from("Player"),
        Line::from(cards),
        Line::from(format!("Score: {}", view.player_score)),
    ];

    let block = Block::default().title("Player").borders(Borders::ALL);

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_status(f: &mut Frame, area: ratatui::layout::Rect, view: &BlackjackView) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        view.phase.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    )));

    if view.result != GameResult::Pending {
        lines.push(Line::from(view.result.to_string()));
    }

    lines.push(Line::from(""));

    let controls = view
        .available_actions
        .iter()
        .map(|c| match c {
            PlayerAction::Hit => "[H] Hit",
            PlayerAction::Stay => "[S] Stay",
            PlayerAction::Double => "[D] Double",
            PlayerAction::Split => "[P] Split",
            PlayerAction::NewRound => "[N] New Round",
            PlayerAction::Quit => "[Q] Quit",
        })
        .collect::<Vec<_>>()
        .join("   ");

    lines.push(Line::from(Span::styled(
        controls,
        Style::default().fg(Color::Yellow),
    )));

    let block = Block::default().title("Status").borders(Borders::ALL);

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_cards(cards: &[VisibleCard]) -> Line<'static> {
    let spans = cards.iter().map(|card| match card {
        VisibleCard::FaceUp(card) => {
            Span::styled(format!("[{}]", card), Style::default().fg(Color::White))
        }
        VisibleCard::FaceDown => {
            Span::styled("[##]".to_string(), Style::default().fg(Color::DarkGray))
        }
    });

    Line::from(spans.collect::<Vec<_>>())
}

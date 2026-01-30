use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use card_games::game::blackjack::{
    types::GameResult,
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
        .split(f.size());

    draw_dealer(f, chunks[0], view);
    draw_player(f, chunks[1], view);
    draw_status(f, chunks[2], view);
}

fn draw_dealer(f: &mut Frame, area: ratatui::layout::Rect, view: &BlackjackView) {
    let cards = render_cards(&view.dealer_cards);

    let score = match view.dealer_score {
        Some(score) => format!("Score: {}", score),
        None => "Score: ?".to_string(),
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

    let controls = match (view.can_hit, view.can_stay) {
        (true, true) => "[H] Hit   [S] Stay   [Q] Quit",
        _ => "[Q] Quit",
    };

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

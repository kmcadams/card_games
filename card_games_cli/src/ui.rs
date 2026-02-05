use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
            Constraint::Min(7),    // player
            Constraint::Min(3),    // status / controls
        ])
        .split(f.area());

    draw_dealer(f, chunks[0], view);
    draw_player(f, chunks[1], view);
    draw_status(f, chunks[2], view);
}

fn draw_dealer(f: &mut Frame, area: ratatui::layout::Rect, view: &BlackjackView) {
    let cards = Line::from(render_cards(&view.dealer_cards));

    let score = match (view.dealer_visible_score, view.dealer_has_hidden_card) {
        (Some(score), true) => format!("Score: {} + ?", score),
        (Some(score), false) => format!("Score: {}", score),
        _ => "Score: ?".to_string(),
    };

    let text = vec![Line::from("Dealer"), Line::from(cards), Line::from(score)];

    let block = Block::default().title("Dealer").borders(Borders::ALL);

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_player(f: &mut Frame, area: Rect, view: &BlackjackView) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Player",
        Style::default().add_modifier(Modifier::BOLD),
    )));

    for (i, hand) in view.player_hands.iter().enumerate() {
        let is_active = i == view.active_hand_index;

        let prefix = if is_active { "> " } else { "  " };
        let style = if is_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(Span::styled(
            format!("{}Hand {}  Bet: ${}", prefix, i + 1, hand.bet_amount),
            style,
        )));

        let mut card_spans = Vec::new();
        card_spans.push(Span::raw("    "));
        card_spans.extend(render_cards(&hand.cards));

        lines.push(Line::from(card_spans));

        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("Score: {}", hand.score), style),
        ]));

        lines.push(Line::raw(""));
    }

    let block = Block::default().title("Player").borders(Borders::ALL);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_status(f: &mut Frame, area: Rect, view: &BlackjackView) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // phase + result
            Constraint::Length(7), // bank + bet
            Constraint::Min(3),    // controls
        ])
        .split(area);

    draw_phase_and_result(f, chunks[0], view);
    draw_bank(f, chunks[1], view);
    draw_controls(f, chunks[2], view);
}

fn draw_phase_and_result(f: &mut Frame, area: Rect, view: &BlackjackView) {
    let mut lines = vec![Line::from(Span::styled(
        view.phase.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    if view.result != GameResult::Pending {
        lines.push(Line::from(view.result.to_string()));
    }

    let block = Block::default().borders(Borders::ALL).title("Game");

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_bank(f: &mut Frame, area: Rect, view: &BlackjackView) {
    let total_bet: u32 = view.player_hands.iter().map(|h| h.bet_amount).sum();

    let line = Line::from(vec![
        Span::styled("Balance: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${}", view.bank_balance),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("Bet: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("${}", total_bet),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let block = Block::default().borders(Borders::ALL).title("Bank");

    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_controls(f: &mut Frame, area: Rect, view: &BlackjackView) {
    let mut controls = view
        .available_actions
        .iter()
        .map(|c| match c {
            PlayerAction::Hit => "[H] Hit",
            PlayerAction::Stay => "[S] Stay",
            PlayerAction::Double => "[D] Double",
            PlayerAction::Split => "[P] Split",
        })
        .collect::<Vec<_>>();

    if view.can_start_new_round {
        controls.push("[N] New Round");
    }
    controls.push("[Q] Quit");

    let controls = controls.join("   ");

    let block = Block::default().borders(Borders::ALL).title("Controls");

    f.render_widget(
        Paragraph::new(controls)
            .style(Style::default().fg(Color::Cyan))
            .block(block),
        area,
    );
}

fn render_cards(cards: &[VisibleCard]) -> Vec<Span<'static>> {
    cards
        .iter()
        .map(|card| match card {
            VisibleCard::FaceUp(card) => {
                Span::styled(format!("[{}]", card), Style::default().fg(Color::White))
            }
            VisibleCard::FaceDown => {
                Span::styled("[##]".to_string(), Style::default().fg(Color::DarkGray))
            }
        })
        .collect()
}

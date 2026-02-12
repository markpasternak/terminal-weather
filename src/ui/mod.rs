pub mod layout;
pub mod particles;
pub mod theme;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::state::{AppMode, AppState},
    cli::Cli,
    resilience::freshness::FreshnessState,
};

pub fn render(frame: &mut Frame, state: &AppState, cli: &Cli) {
    let area = frame.area();

    if area.width < 30 || area.height < 15 {
        let warning = Paragraph::new("Terminal too small. Resize to at least 30x15.")
            .block(Block::default().borders(Borders::ALL).title("atmos-tui"));
        frame.render_widget(warning, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .split(area);

    widgets::hero::render(frame, chunks[0], state, cli);
    widgets::hourly::render(frame, chunks[1], state, cli);
    widgets::daily::render(frame, chunks[2], state, cli);

    render_status_badge(frame, area, state);

    if state.mode == AppMode::SelectingLocation {
        widgets::selector::render(frame, centered_rect(70, 60, area), state);
    }
}

fn render_status_badge(frame: &mut Frame, area: Rect, state: &AppState) {
    let label = match state.refresh_meta.state {
        FreshnessState::Fresh => None,
        FreshnessState::Stale => Some(("⚠ stale", Color::Yellow)),
        FreshnessState::Offline => Some(("⚠ offline", Color::LightRed)),
    };

    if let Some((text, color)) = label {
        let width = (text.chars().count() as u16 + 2).min(area.width);
        let badge_area = Rect {
            x: area.right().saturating_sub(width + 1),
            y: area.y,
            width,
            height: 1,
        };
        let badge = Paragraph::new(Line::from(text)).style(
            Style::default()
                .fg(color)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(badge, badge_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

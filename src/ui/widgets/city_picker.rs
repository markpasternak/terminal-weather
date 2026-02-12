use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::state::AppState,
    ui::theme::{detect_color_capability, theme_for},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let (category, is_day) = state
        .weather
        .as_ref()
        .map(|w| {
            (
                crate::domain::weather::weather_code_to_category(w.current.weather_code),
                w.current.is_day,
            )
        })
        .unwrap_or((crate::domain::weather::WeatherCategory::Unknown, false));
    let theme = theme_for(
        category,
        is_day,
        detect_color_capability(),
        state.settings.theme,
    );
    let panel_style = Style::default()
        .fg(theme.popup_text)
        .bg(theme.popup_surface);

    let block = Block::default()
        .title("Locations")
        .borders(Borders::ALL)
        .style(panel_style)
        .border_style(
            Style::default()
                .fg(theme.popup_border)
                .bg(theme.popup_surface),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(2),
    ])
    .split(inner);

    let query = if state.city_query.is_empty() {
        "Type a city and press Enter (or use history)"
    } else {
        state.city_query.as_str()
    };
    let query_line = Paragraph::new(vec![Line::from(vec![
        Span::styled("Search: ", Style::default().fg(theme.popup_muted_text)),
        Span::styled(
            query,
            Style::default()
                .fg(theme.popup_text)
                .add_modifier(Modifier::BOLD),
        ),
    ])])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.popup_border)),
    );
    frame.render_widget(query_line, chunks[0]);

    let mut items = state
        .settings
        .recent_locations
        .iter()
        .take(9)
        .enumerate()
        .map(|(idx, saved)| ListItem::new(format!("{}. {}", idx + 1, saved.display_name())))
        .collect::<Vec<_>>();
    if !items.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "Clear all recent locations",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )])));
    }

    let mut list_state = ListState::default().with_selected(Some(
        state
            .city_history_selected
            .min(items.len().saturating_sub(1)),
    ));
    let list = if items.is_empty() {
        List::new(vec![ListItem::new("No recent cities yet")])
    } else {
        List::new(items)
    }
    .style(panel_style)
    .highlight_style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("› ")
    .block(
        Block::default()
            .title("Recent (1-9 quick switch · Del clear all)")
            .borders(Borders::NONE),
    );
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let status_text = state.city_status.as_deref().unwrap_or(
        "Shortcuts: Enter search/switch · ↑/↓ history · Del clear all · Backspace edit · Esc close",
    );
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.popup_muted_text));
    frame.render_widget(status, chunks[2]);
}

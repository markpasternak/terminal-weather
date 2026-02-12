use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
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

    let block = Block::default()
        .title("Settings")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(inner);

    let items = state
        .settings_entries()
        .into_iter()
        .map(|entry| {
            let label = if entry.editable {
                format!("{:<16} {}", entry.label, entry.value)
            } else {
                format!("{:<16} [{}]", entry.label, entry.value)
            };
            ListItem::new(Line::from(label))
        })
        .collect::<Vec<_>>();

    let mut list_state = ListState::default().with_selected(Some(state.settings_selected));
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let info =
        Paragraph::new("Use ↑/↓ to select, ←/→ to change, Enter to apply action, s to close")
            .style(Style::default().fg(theme.muted_text));
    frame.render_widget(info, chunks[1]);

    if let Some(path) = &state.last_error
        && path.contains("save settings")
    {
        let warn = Paragraph::new(path.clone()).style(Style::default().fg(theme.warning));
        frame.render_widget(warn, chunks[2]);
    }
}

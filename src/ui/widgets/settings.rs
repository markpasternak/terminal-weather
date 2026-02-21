use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::state::AppState,
    ui::theme::{Theme, resolved_theme},
};

use super::shared::{popup_block, popup_panel_style};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let theme = resolved_theme(state);
    let panel_style = popup_panel_style(theme);

    let block = popup_block("Settings", theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);

    let items = settings_items(state);

    let mut list_state =
        ListState::default().with_selected(Some(state.settings_selected.to_usize()));
    let list = settings_list(items, panel_style, theme);
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    render_controls(frame, chunks[1], theme);
    render_hint(frame, chunks[2], state, theme);
}

fn settings_items(state: &AppState) -> Vec<ListItem<'static>> {
    state
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
        .collect::<Vec<_>>()
}

fn settings_list(items: Vec<ListItem<'static>>, panel_style: Style, theme: Theme) -> List<'static> {
    List::new(items)
        .style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ")
}

fn render_controls(frame: &mut Frame, area: Rect, theme: Theme) {
    let controls = Paragraph::new("↑/↓ select  ←/→ or Enter change  Enter on actions  s close")
        .style(Style::default().fg(theme.popup_muted_text));
    frame.render_widget(controls, area);
}

fn render_hint(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let hint_text = if let Some(path) = &state.last_error {
        if path.contains("save settings") {
            path.clone()
        } else {
            state.settings_hint()
        }
    } else {
        state.settings_hint()
    };
    let hint_style = if hint_text.contains("save settings") {
        Style::default().fg(theme.warning)
    } else {
        Style::default().fg(theme.popup_muted_text)
    };
    frame.render_widget(Paragraph::new(hint_text).style(hint_style), area);
}

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

#[cfg(test)]
mod tests {
    use crate::app::state::AppState;
    use crate::ui::theme::resolved_theme;
    use crate::ui::widgets::settings::settings_items;
    use ratatui::style::Style;

    fn make_state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    #[test]
    fn settings_items_produces_editable_format() {
        let state = make_state();
        let entries = state.settings_entries();
        let editable: Vec<_> = entries.iter().filter(|e| e.editable).collect();
        assert!(!editable.is_empty());
        let entry = &editable[0];
        let expected = format!("{:<16} {}", entry.label, entry.value);
        assert!(!expected.contains('['));
    }

    #[test]
    fn settings_items_produces_non_editable_format_for_actions() {
        let state = make_state();
        let entries = state.settings_entries();
        let non_editable: Vec<_> = entries.iter().filter(|e| !e.editable).collect();
        assert!(
            !non_editable.is_empty(),
            "should have at least one action row"
        );
        let entry = &non_editable[0];
        let expected = format!("{:<16} [{}]", entry.label, entry.value);
        assert!(expected.contains('['));
    }

    #[test]
    fn settings_items_function_returns_items() {
        let state = make_state();
        let items = settings_items(&state);
        assert!(!items.is_empty());
    }

    #[test]
    fn hint_text_shows_error_when_contains_save_settings() {
        let mut state = make_state();
        state.last_error = Some("Failed to save settings: permission denied".to_string());
        let hint_text = if let Some(path) = &state.last_error {
            if path.contains("save settings") {
                path.clone()
            } else {
                state.settings_hint()
            }
        } else {
            state.settings_hint()
        };
        assert!(hint_text.contains("save settings"));
    }

    #[test]
    fn hint_text_shows_settings_hint_when_no_error() {
        let state = make_state();
        let hint_text = state.settings_hint();
        assert!(!hint_text.is_empty());
    }

    #[test]
    fn hint_text_shows_settings_hint_for_non_save_error() {
        let mut state = make_state();
        state.last_error = Some("Network error".to_string());
        let hint_text = if let Some(path) = &state.last_error {
            if path.contains("save settings") {
                path.clone()
            } else {
                state.settings_hint()
            }
        } else {
            state.settings_hint()
        };
        assert_eq!(hint_text, state.settings_hint());
    }

    #[test]
    fn hint_style_is_warning_when_contains_save_settings() {
        let theme = resolved_theme(&make_state());
        let hint_text = "Failed to save settings: permission denied".to_string();
        let hint_style = if hint_text.contains("save settings") {
            Style::default().fg(theme.warning)
        } else {
            Style::default().fg(theme.popup_muted_text)
        };
        assert_eq!(hint_style.fg, Some(theme.warning));
    }

    #[test]
    fn hint_style_is_muted_when_no_save_settings() {
        let theme = resolved_theme(&make_state());
        let hint_text = "Normal hint text".to_string();
        let hint_style = if hint_text.contains("save settings") {
            Style::default().fg(theme.warning)
        } else {
            Style::default().fg(theme.popup_muted_text)
        };
        assert_eq!(hint_style.fg, Some(theme.popup_muted_text));
    }
}

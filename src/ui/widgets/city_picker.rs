#![allow(clippy::too_many_lines)]

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::{settings::RecentLocation, state::AppState},
    ui::theme::{Theme, resolved_theme},
};

use super::shared::{popup_block, popup_panel_style};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    let theme = resolved_theme(state);
    let panel_style = popup_panel_style(theme);

    let block = popup_block("Locations", theme, panel_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(2),
    ])
    .split(inner);

    render_query_line(frame, chunks[0], state, theme);
    let items = recent_city_items(state, theme);
    let mut list_state =
        ListState::default().with_selected(Some(selected_recent_index(state, &items)));
    let list = recent_city_list(items, panel_style, theme);
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    render_status_line(frame, chunks[2], state, theme);
}

fn render_query_line(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let (query, style) = if state.city_query.is_empty() {
        (
            "Type a city and press Enter (or use history)",
            Style::default().fg(theme.popup_muted_text),
        )
    } else {
        (
            state.city_query.as_str(),
            Style::default()
                .fg(theme.popup_text)
                .add_modifier(Modifier::BOLD),
        )
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.popup_border));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let [input_area, count_area] = Layout::horizontal([
        Constraint::Min(10),
        Constraint::Length(10), // Space for (50/50)
    ])
    .areas(inner_area);

    let query_line = Paragraph::new(vec![Line::from(vec![
        Span::styled("Search: ", Style::default().fg(theme.popup_muted_text)),
        Span::styled(query, style),
    ])]);
    frame.render_widget(query_line, input_area);

    let count = state.city_query.chars().count();
    let max = 50;
    let count_style = if count >= max {
        Style::default().fg(theme.warning)
    } else {
        Style::default().fg(theme.popup_muted_text)
    };

    let counter = Paragraph::new(Line::from(format!("({count}/{max})")))
        .style(count_style)
        .alignment(Alignment::Right);
    frame.render_widget(counter, count_area);

    let prefix_width = "Search: ".len() as u16;
    let query_width = if state.city_query.is_empty() {
        0
    } else {
        Line::from(state.city_query.as_str()).width() as u16
    };
    // Ensure cursor doesn't drift if layout shifts, though with Borders::BOTTOM
    // inner_area.x usually equals area.x
    frame.set_cursor_position((input_area.x + prefix_width + query_width, input_area.y));
}

fn recent_city_items(state: &AppState, theme: Theme) -> Vec<ListItem<'static>> {
    let mut items = state
        .settings
        .recent_locations
        .iter()
        .take(9)
        .enumerate()
        .map(|(idx, saved)| ListItem::new(format_recent_location(idx, saved, state)))
        .collect::<Vec<_>>();
    if !items.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "Clear all recent locations",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )])));
    }
    items
}

fn selected_recent_index(state: &AppState, items: &[ListItem<'_>]) -> usize {
    state
        .city_history_selected
        .min(items.len().saturating_sub(1))
}

fn recent_city_list(
    items: Vec<ListItem<'static>>,
    panel_style: Style,
    theme: Theme,
) -> List<'static> {
    let body = if items.is_empty() {
        let text = Line::from(vec![
            Span::styled(
                "No recent cities. Type to search · ",
                Style::default().fg(theme.popup_muted_text),
            ),
            Span::styled(
                "Esc",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" close", Style::default().fg(theme.popup_muted_text)),
        ]);
        List::new(vec![ListItem::new(text)])
    } else {
        List::new(items)
    };
    body.style(panel_style)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ")
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled("Recent (", Style::default().fg(theme.popup_muted_text)),
                    Span::styled(
                        "1-9",
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " quick switch · ",
                        Style::default().fg(theme.popup_muted_text),
                    ),
                    Span::styled(
                        "Del",
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" clear all)", Style::default().fg(theme.popup_muted_text)),
                ]))
                .borders(Borders::NONE),
        )
}

fn render_status_line(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let status = Paragraph::new(city_picker_state_line(state, theme));
    frame.render_widget(status, area);
}

fn format_recent_location(index: usize, saved: &RecentLocation, state: &AppState) -> String {
    let timezone = saved.timezone.as_deref().unwrap_or("--");
    let region = match (&saved.country, &saved.admin1) {
        (Some(country), Some(admin)) => format!("{country} · {admin}"),
        (Some(country), None) => country.clone(),
        (None, Some(admin)) => admin.clone(),
        (None, None) => "--".to_string(),
    };
    let marker = if is_selected_location(saved, state) {
        "* "
    } else {
        ""
    };
    format!(
        "{}. {}{} · {} · TZ {} · {:.2}, {:.2}",
        index + 1,
        marker,
        saved.display_name(),
        region,
        timezone,
        saved.latitude,
        saved.longitude
    )
}

fn city_picker_state_line(state: &AppState, theme: Theme) -> Line<'static> {
    let muted = Style::default().fg(theme.popup_muted_text);
    let key = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);

    let detail = state.city_status.as_deref().unwrap_or(
        "Enter search/switch · ↑/↓ history · Del clear all · Backspace edit · Esc close",
    );
    let kind = city_status_kind(detail);

    let mut spans = vec![Span::styled(format!("State: {kind} · "), muted)];

    if detail.contains("Enter search/switch") {
        spans.extend(vec![
            Span::styled("Enter", key),
            Span::styled(" search/switch · ", muted),
            Span::styled("↑/↓", key),
            Span::styled(" history · ", muted),
            Span::styled("Del", key),
            Span::styled(" clear all · ", muted),
            Span::styled("Backspace", key),
            Span::styled(" edit · ", muted),
            Span::styled("Esc", key),
            Span::styled(" close", muted),
        ]);
    } else {
        spans.push(Span::styled(format!("{detail} · "), muted));
        spans.push(Span::styled("Esc", key));
        spans.push(Span::styled(" close", muted));
    }

    Line::from(spans)
}

fn city_status_kind(status: &str) -> &'static str {
    if status.contains("Searching") {
        "Searching"
    } else if status.contains("No results") {
        "No results"
    } else if status.contains("Ambiguous") {
        "Ambiguous"
    } else if status.contains("Failed") {
        "Failed"
    } else {
        "Ready"
    }
}

fn is_selected_location(saved: &RecentLocation, state: &AppState) -> bool {
    state.selected_location.as_ref().is_some_and(|selected| {
        let selected_recent = RecentLocation::from_location(selected);
        saved.same_place(&selected_recent)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{settings::RecentLocation, state::AppState};

    fn make_state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    fn sample_recent(name: &str, lat: f64, lon: f64) -> RecentLocation {
        RecentLocation {
            name: name.to_string(),
            country: Some("Sweden".to_string()),
            admin1: None,
            latitude: lat,
            longitude: lon,
            timezone: Some("Europe/Stockholm".to_string()),
        }
    }

    #[test]
    fn format_recent_location_includes_all_fields() {
        let loc = sample_recent("Stockholm", 59.33, 18.07);
        let state = make_state();
        let text = format_recent_location(0, &loc, &state);
        assert!(text.contains("Stockholm"), "got: {text}");
        assert!(text.contains("59.33"), "got: {text}");
        assert!(text.contains("18.07"), "got: {text}");
        assert!(text.contains("Europe/Stockholm"), "got: {text}");
        assert!(text.starts_with("1."), "got: {text}");
    }

    #[test]
    fn format_recent_location_second_entry_has_index_two() {
        let loc = sample_recent("Gothenburg", 57.70, 11.97);
        let state = make_state();
        let text = format_recent_location(1, &loc, &state);
        assert!(text.starts_with("2."), "got: {text}");
    }

    #[test]
    fn selected_recent_index_clamps_to_items_count() {
        use ratatui::widgets::ListItem;
        let items = vec![ListItem::new("a"), ListItem::new("b")];
        let mut state = make_state();
        state.city_history_selected = 99;
        let idx = selected_recent_index(&state, &items);
        assert_eq!(idx, 1); // clamped to len-1
    }

    #[test]
    fn selected_recent_index_zero_for_empty_items() {
        use ratatui::widgets::ListItem;
        let items: Vec<ListItem<'_>> = vec![];
        let state = make_state();
        let idx = selected_recent_index(&state, &items);
        assert_eq!(idx, 0);
    }

    #[test]
    fn is_selected_location_returns_true_when_match() {
        let mut state = make_state();
        state.selected_location = Some(crate::test_support::stockholm_location());
        let recent = RecentLocation::from_location(&state.selected_location.clone().unwrap());
        assert!(is_selected_location(&recent, &state));
    }

    #[test]
    fn is_selected_location_returns_false_when_no_selected() {
        let state = make_state();
        let recent = RecentLocation {
            name: "Stockholm".to_string(),
            latitude: 59.33,
            longitude: 18.07,
            country: Some("Sweden".to_string()),
            admin1: None,
            timezone: None,
        };
        assert!(!is_selected_location(&recent, &state));
    }

    #[test]
    fn recent_city_items_empty_returns_empty_vec() {
        let state = make_state();
        let items = recent_city_items(&state, resolved_theme(&state));
        assert!(items.is_empty());
    }

    #[test]
    fn recent_city_items_non_empty_includes_clear_option() {
        let mut state = make_state();
        state.settings.recent_locations.push(RecentLocation {
            name: "Stockholm".to_string(),
            latitude: 59.33,
            longitude: 18.07,
            country: Some("Sweden".to_string()),
            admin1: None,
            timezone: Some("Europe/Stockholm".to_string()),
        });
        let items = recent_city_items(&state, resolved_theme(&state));
        assert!(items.len() >= 2);
    }

    #[test]
    fn recent_city_list_empty_items_shows_no_recent_message() {
        let theme = resolved_theme(&make_state());
        let panel_style = Style::default();
        let items: Vec<ListItem<'static>> = vec![];
        let list = recent_city_list(items, panel_style, theme);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn recent_city_list_non_empty_items() {
        let theme = resolved_theme(&make_state());
        let panel_style = Style::default();
        let items = vec![ListItem::new("Test")];
        let list = recent_city_list(items, panel_style, theme);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn format_recent_location_with_marker_when_selected() {
        let mut state = make_state();
        state.selected_location = Some(crate::test_support::stockholm_location());
        let recent = RecentLocation::from_location(&state.selected_location.clone().unwrap());
        let text = format_recent_location(0, &recent, &state);
        assert!(text.contains('*'));
    }

    #[test]
    fn format_recent_location_without_timezone_shows_placeholder() {
        let state = make_state();
        let recent = RecentLocation {
            name: "Tokyo".to_string(),
            latitude: 35.68,
            longitude: 139.69,
            country: Some("Japan".to_string()),
            admin1: None,
            timezone: None,
        };
        let text = format_recent_location(0, &recent, &state);
        assert!(text.contains("--"));
    }

    #[test]
    fn render_status_line_uses_custom_status_when_present() {
        let mut state = make_state();
        state.city_status = Some("Searching...".to_string());
        let theme = resolved_theme(&state);
        let status_line = city_picker_state_line(&state, theme);
        let status_text: String = status_line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(status_text.contains("State: Searching"));
    }

    #[test]
    fn city_status_kind_detects_failure() {
        assert_eq!(city_status_kind("Failed to fetch weather"), "Failed");
    }

    #[test]
    fn render_sets_cursor_position_when_query_active() {
        use ratatui::{
            Terminal,
            backend::{Backend, TestBackend},
            layout::Position,
        };
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = make_state();
        state.city_picker_open = true;
        state.city_query = "Lon".to_string();

        terminal
            .draw(|f| {
                let area = f.area();
                super::render(f, area, &state);
            })
            .unwrap();

        // Expected cursor X position:
        // Area (0,0) -> Border inner (1,1) -> Chunks[0] (1,1)
        // "Search: " (8 chars) + "Lon" (3 chars) = 11 chars offset.
        // X = 1 + 11 = 12.
        // Y = 1.
        let cursor_pos = terminal.backend_mut().get_cursor_position().unwrap();
        assert_eq!(cursor_pos, Position::new(12, 1));
    }

    #[test]
    fn render_uses_muted_style_for_empty_query_placeholder() {
        use ratatui::{Terminal, backend::TestBackend, style::Modifier};
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = make_state();
        state.city_picker_open = true;
        state.city_query = String::new();

        terminal
            .draw(|f| {
                let area = f.area();
                super::render(f, area, &state);
            })
            .unwrap();

        // "Search: " (8 chars). Inner area starts at (1,1).
        // "Search: " is at x=1..9.
        // Placeholder "Type a city..." starts at x=9.
        let buffer = terminal.backend().buffer();
        let cell = &buffer[(9, 1)];

        // Placeholder text should be muted and not bold (unlike user input)
        assert!(
            !cell.modifier.contains(Modifier::BOLD),
            "Placeholder should not be bold"
        );
    }
}

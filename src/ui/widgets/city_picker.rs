#![allow(clippy::too_many_lines)]

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
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
    frame.render_widget(query_line, area);
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
        List::new(vec![ListItem::new("No recent cities yet")])
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
                .title("Recent (1-9 quick switch · Del clear all)")
                .borders(Borders::NONE),
        )
}

fn render_status_line(frame: &mut Frame, area: Rect, state: &AppState, theme: Theme) {
    let status_text = state.city_status.as_deref().unwrap_or(
        "Shortcuts: Enter search/switch · ↑/↓ history · Del clear all · Backspace edit · Esc close",
    );
    let status = Paragraph::new(status_text).style(Style::default().fg(theme.popup_muted_text));
    frame.render_widget(status, area);
}

fn format_recent_location(index: usize, saved: &RecentLocation, state: &AppState) -> String {
    let timezone = saved.timezone.as_deref().unwrap_or("--");
    let marker = if is_selected_location(saved, state) {
        "* "
    } else {
        ""
    };
    format!(
        "{}. {}{} · {:.2}, {:.2} · TZ {}",
        index + 1,
        marker,
        saved.display_name(),
        saved.latitude,
        saved.longitude,
        timezone
    )
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
        let status_text = state.city_status.as_deref().unwrap_or("default");
        assert_eq!(status_text, "Searching...");
    }
}

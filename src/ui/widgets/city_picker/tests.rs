use ratatui::{
    Terminal,
    backend::{Backend, TestBackend},
    layout::Position,
    style::{Modifier, Style},
    widgets::ListItem,
};

use super::{
    city_picker_state_line, city_status_kind, format_recent_location, is_selected_location,
    recent_city_items, recent_city_list, render, selected_recent_index,
};
use crate::{
    app::{settings::RecentLocation, state::AppState},
    ui::theme::resolved_theme,
};

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
    let items = vec![ListItem::new("a"), ListItem::new("b")];
    let mut state = make_state();
    state.city_history_selected = 99;
    let idx = selected_recent_index(&state, &items);
    assert_eq!(idx, 1);
}

#[test]
fn selected_recent_index_zero_for_empty_items() {
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
        .map(|span| span.content.as_ref())
        .collect();
    assert!(status_text.contains("State: Searching"));
}

#[test]
fn city_status_kind_detects_failure() {
    assert_eq!(city_status_kind("Failed to fetch weather"), "Failed");
}

#[test]
fn render_sets_cursor_position_when_query_active() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = make_state();
    state.city_picker_open = true;
    state.city_query = "Lon".to_string();

    terminal
        .draw(|frame| {
            let area = frame.area();
            render(frame, area, &state);
        })
        .unwrap();

    let cursor_pos = terminal.backend_mut().get_cursor_position().unwrap();
    assert_eq!(cursor_pos, Position::new(12, 1));
}

#[test]
fn render_uses_muted_style_for_empty_query_placeholder() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = make_state();
    state.city_picker_open = true;
    state.city_query = String::new();

    terminal
        .draw(|frame| {
            let area = frame.area();
            render(frame, area, &state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let cell = &buffer[(9, 1)];

    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "Placeholder should not be bold"
    );
}

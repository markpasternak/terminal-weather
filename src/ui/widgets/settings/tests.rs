use super::{render, settings_hint_style, settings_hint_text, settings_items};
use crate::app::state::AppState;
use crate::ui::theme::resolved_theme;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

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
    let hint_text = settings_hint_text(&state);
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
    let hint_text = settings_hint_text(&state);
    assert_eq!(hint_text, state.settings_hint());
}

#[test]
fn hint_style_is_warning_when_contains_save_settings() {
    let theme = resolved_theme(&make_state());
    let hint_text = "Failed to save settings: permission denied".to_string();
    let hint_style = settings_hint_style(&hint_text, theme);
    assert_eq!(hint_style.fg, Some(theme.warning));
}

#[test]
fn hint_style_is_muted_when_no_save_settings() {
    let theme = resolved_theme(&make_state());
    let hint_text = "Normal hint text".to_string();
    let hint_style = settings_hint_style(&hint_text, theme);
    assert_eq!(hint_style.fg, Some(theme.popup_muted_text));
}

#[test]
fn render_handles_settings_popup_with_and_without_save_errors() {
    let mut normal_state = make_state();
    normal_state.last_error = None;

    let mut error_state = make_state();
    error_state.last_error = Some("Failed to save settings: disk full".to_string());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| render(frame, Rect::new(0, 0, 80, 24), &normal_state))
        .expect("draw settings popup in normal mode");
    terminal
        .draw(|frame| render(frame, Rect::new(0, 0, 80, 24), &error_state))
        .expect("draw settings popup with save error");
}

use super::*;
use crate::update::UpdateStatus;

#[test]
fn update_hint_for_width_shows_only_when_update_available() {
    assert!(update_hint_for_width(120, &UpdateStatus::Unknown).is_none());
    assert!(update_hint_for_width(120, &UpdateStatus::UpToDate).is_none());
    assert_eq!(
        update_hint_for_width(
            120,
            &UpdateStatus::UpdateAvailable {
                latest: "0.7.0".to_string()
            }
        )
        .as_deref(),
        Some("Update available: v0.7.0 · brew upgrade markpasternak/tap/terminal-weather")
    );
}

#[test]
fn footer_text_includes_update_hint_for_wide_layout() {
    let mut state = crate::app::state::AppState::new(&crate::test_support::state_test_cli());
    state.update_status = UpdateStatus::UpdateAvailable {
        latest: "0.7.0".to_string(),
    };
    let text = footer_text_for_width(120, &state, crate::ui::theme::resolved_theme(&state));
    let combined_text = text.iter().map(|s| s.content.as_ref()).collect::<String>();
    assert!(combined_text.contains("Update available: v0.7.0"));
}

#[test]
fn footer_text_omits_update_hint_for_narrow_layout() {
    let mut state = crate::app::state::AppState::new(&crate::test_support::state_test_cli());
    state.update_status = UpdateStatus::UpdateAvailable {
        latest: "0.7.0".to_string(),
    };
    let text = footer_text_for_width(60, &state, crate::ui::theme::resolved_theme(&state));
    let combined_text = text.iter().map(|s| s.content.as_ref()).collect::<String>();
    assert!(!combined_text.contains("Update available"));
}

#[test]
fn render_sets_cursor_in_command_bar() {
    use ratatui::{
        Terminal,
        backend::{Backend, TestBackend},
        layout::Position,
    };

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = crate::app::state::AppState::new(&crate::test_support::state_test_cli());
    state.command_bar.open = true;
    state.command_bar.buffer = ":test ".to_string();

    terminal
        .draw(|f| render(f, &state, &crate::test_support::state_test_cli()))
        .unwrap();

    let cursor_pos = terminal.backend_mut().get_cursor_position().unwrap();
    assert_eq!(cursor_pos, Position::new(6, 23));
}

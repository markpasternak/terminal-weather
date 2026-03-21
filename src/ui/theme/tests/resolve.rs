use crate::{
    app::state::AppState,
    cli::ThemeArg,
    test_support::{sample_bundle, state_test_cli},
    ui::theme::resolve::theme_preview,
};

#[test]
fn theme_preview_auto_with_bundle() {
    let mut state = AppState::new(&state_test_cli());
    state.settings.theme = ThemeArg::Auto;
    state.weather = Some(sample_bundle());

    let preview = theme_preview(&state);
    assert!(preview.starts_with("Auto:"));
}

#[test]
fn theme_preview_auto_without_bundle() {
    let mut state = AppState::new(&state_test_cli());
    state.settings.theme = ThemeArg::Auto;
    state.weather = None;

    let preview = theme_preview(&state);
    assert_eq!(preview, "Auto: cinematic weather-aware palette");
}

#[test]
fn theme_preview_explicit_ignores_bundle() {
    let mut state = AppState::new(&state_test_cli());
    state.settings.theme = ThemeArg::Nord;
    state.weather = None;

    let preview_no_bundle = theme_preview(&state);
    assert!(preview_no_bundle.starts_with("Nord:"));

    state.weather = Some(sample_bundle());
    let preview_with_bundle = theme_preview(&state);

    assert_eq!(preview_no_bundle, preview_with_bundle);
}

#[test]
fn theme_preview_covers_multiple_explicit_themes() {
    let mut state = AppState::new(&state_test_cli());

    state.settings.theme = ThemeArg::TokyoNightStorm;
    assert!(theme_preview(&state).starts_with("Tokyo Night Storm:"));

    state.settings.theme = ThemeArg::Dracula;
    assert!(theme_preview(&state).starts_with("Dracula:"));
}

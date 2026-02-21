use super::super::test_support::{sample_bundle, test_cli};
use super::{compass, fetch_context_line, format_visibility, freshness_flag, last_updated_label};
use crate::{
    app::state::AppState,
    cli::ThemeArg,
    domain::weather::RefreshMetadata,
    resilience::freshness::FreshnessState,
    ui::theme::{ColorCapability, theme_for},
};
use chrono::{Duration, Utc};

#[test]
fn freshness_flag_warns_on_stale() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Stale;
    let theme = theme_for(
        crate::domain::weather::WeatherCategory::Unknown,
        false,
        ColorCapability::TrueColor,
        ThemeArg::Auto,
    );

    let flag = freshness_flag(&state, theme).expect("stale flag");
    assert_eq!(flag.0, "⚠ stale");
}

#[test]
fn last_updated_label_includes_timezone() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta = RefreshMetadata {
        last_success: Some(Utc::now() - Duration::minutes(3)),
        ..RefreshMetadata::default()
    };
    let weather = sample_bundle();
    let label = last_updated_label(&state, &weather);
    assert!(label.contains("TZ Europe/Stockholm"));
}

#[test]
fn compass_rounds_directions() {
    assert_eq!(compass(0.0), "N");
    assert_eq!(compass(44.0), "NE");
    assert_eq!(compass(225.0), "SW");
}

#[test]
fn format_visibility_formats_km() {
    assert_eq!(format_visibility(12_345.0), "12.3km");
    assert_eq!(format_visibility(20_100.0), "20km");
    assert_eq!(format_visibility(-1.0), "--");
}

#[test]
fn fetch_context_line_shows_retry_when_available() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Offline;
    state.last_error = Some("network timeout".to_string());
    state.refresh_meta.schedule_retry_in(30);

    let line = fetch_context_line(&state).expect("fetch context line");
    assert!(line.contains("network timeout"));
    assert!(line.contains("retry in"));
}

#[test]
fn last_updated_label_without_success_uses_placeholder() {
    let state = AppState::new(&test_cli());
    let weather = sample_bundle();
    let label = last_updated_label(&state, &weather);
    assert!(label.starts_with("Last updated: --:-- local"));
    assert!(label.ends_with("City TZ Europe/Stockholm"));
}

#[test]
fn fetch_context_line_is_suppressed_when_fresh() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Fresh;
    state.last_error = Some("transient error".to_string());
    assert!(fetch_context_line(&state).is_none());
}

#[test]
fn fetch_context_line_truncates_long_multiline_errors() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Offline;
    state.last_error = Some(format!(
        "{}\n{}",
        "x".repeat(120),
        "this second line should not appear"
    ));

    let line = fetch_context_line(&state).expect("fetch context line");
    assert!(line.starts_with("Last fetch failed: "));
    assert!(!line.contains("second line"));
    assert!(line.contains('…'));
}

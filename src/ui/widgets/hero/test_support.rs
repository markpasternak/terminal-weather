use crate::{
    app::state::AppState, cli::Cli, domain::weather::ForecastBundle,
    resilience::freshness::FreshnessState,
};

pub(crate) fn test_cli() -> Cli {
    crate::test_support::hero_test_cli()
}

pub(crate) fn sample_bundle() -> ForecastBundle {
    crate::test_support::sample_bundle()
}

pub(crate) fn assert_last_updated_placeholder(
    label: impl Fn(&AppState, &ForecastBundle) -> String,
    expected_prefix: &str,
) {
    let state = AppState::new(&test_cli());
    let weather = sample_bundle();
    let rendered = label(&state, &weather);
    assert!(rendered.starts_with(expected_prefix));
    assert!(rendered.ends_with("City TZ Europe/Stockholm"));
}

pub(crate) fn assert_fetch_context_suppressed_when_fresh(
    fetch_context: impl Fn(&AppState) -> Option<String>,
) {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Fresh;
    state.last_error = Some("transient error".to_string());
    assert!(fetch_context(&state).is_none());
}

pub(crate) fn assert_fetch_context_truncates_long_multiline_errors(
    fetch_context: impl Fn(&AppState) -> Option<String>,
) {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Offline;
    state.last_error = Some(format!(
        "{}\n{}",
        "x".repeat(120),
        "this second line should not appear"
    ));

    let line = fetch_context(&state).expect("fetch context line");
    assert!(line.starts_with("Last fetch failed: "));
    assert!(!line.contains("second line"));
    assert!(line.contains('…'));
}

#![allow(clippy::cast_precision_loss)]

mod common;

use chrono::Utc;
use common::{
    FixtureProfile, assert_fixture_bundle_shape, assert_stockholm_cli_shape,
    fixture_bundle as shared_fixture_bundle,
    fixture_bundle_with_aqi as shared_fixture_bundle_with_aqi, ready_state_with_weather,
    stockholm_cli,
};
use ratatui::{Terminal, backend::TestBackend};
use terminal_weather::{
    app::state::AppState, cli::Cli, domain::weather::HourlyViewMode,
    resilience::freshness::FreshnessState, ui,
};

fn cli() -> terminal_weather::cli::Cli {
    stockholm_cli()
}

fn fixture_bundle(code: u8) -> terminal_weather::domain::weather::ForecastBundle {
    shared_fixture_bundle(FixtureProfile::Snapshot, code)
}

fn fixture_bundle_with_aqi(code: u8) -> terminal_weather::domain::weather::ForecastBundle {
    shared_fixture_bundle_with_aqi(FixtureProfile::Snapshot, code)
}

fn render_to_string(width: u16, height: u16, weather_code: u8) -> String {
    let cli = cli();
    let state = ready_state_with_weather(&cli, fixture_bundle(weather_code));

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_state_to_string(width: u16, height: u16, state: &AppState, cli: &Cli) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, state, cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_with_hourly_mode_to_string(
    width: u16,
    height: u16,
    weather_code: u8,
    mode: HourlyViewMode,
) -> String {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle(weather_code));
    state.settings.hourly_view = mode;
    state.hourly_view_mode = mode;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_help_to_string(width: u16, height: u16, weather_code: u8) -> String {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle(weather_code));
    state.help_open = true;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

#[test]
fn fixture_bundle_shape_contract() {
    let bundle = fixture_bundle(61);
    assert_fixture_bundle_shape(&bundle, 12, 7, 61);
}

#[test]
fn cli_shape_contract_for_snapshot_tests() {
    let cli = cli();
    assert_stockholm_cli_shape(&cli);
}

macro_rules! assert_weather_snapshot {
    ($fn_name:ident, $snapshot:literal, $width:expr, $height:expr, $code:expr) => {
        #[test]
        fn $fn_name() {
            insta::assert_snapshot!($snapshot, render_to_string($width, $height, $code));
        }
    };
}

assert_weather_snapshot!(snapshot_120x40_clear, "120x40_clear", 120, 40, 0);
assert_weather_snapshot!(snapshot_80x24_rain, "80x24_rain", 80, 24, 61);
assert_weather_snapshot!(snapshot_60x20_snow, "60x20_snow", 60, 20, 71);
assert_weather_snapshot!(snapshot_40x15_fog, "40x15_fog", 40, 15, 45);
assert_weather_snapshot!(snapshot_80x24_thunder, "80x24_thunder", 80, 24, 95);
assert_weather_snapshot!(snapshot_19x9_tiny_fallback, "19x9_tiny_fallback", 19, 9, 0);

#[test]
fn snapshot_80x24_rain_with_aqi() {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle_with_aqi(61));
    state.refresh_meta.last_success = Some(Utc::now());
    insta::assert_snapshot!(
        "80x24_rain_with_aqi",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn small_terminal_still_renders_main_ui() {
    let rendered = render_to_string(20, 10, 0);
    assert!(rendered.contains("Current"));
    assert!(!rendered.contains("Terminal too small"));
}

#[test]
fn below_minimum_terminal_shows_resize_guidance() {
    let rendered = render_to_string(19, 9, 0);
    assert!(rendered.contains("terminal-weather"));
    assert!(!rendered.contains("Current"));
}

#[test]
fn snapshot_100x30_help_overlay() {
    insta::assert_snapshot!("100x30_help_overlay", render_help_to_string(100, 30, 61));
}

#[test]
fn snapshot_80x24_stale_retry_badge() {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle(61));
    state.refresh_meta.state = FreshnessState::Stale;
    state.refresh_meta.last_success = Some(Utc::now() - chrono::Duration::minutes(12));
    state.refresh_meta.schedule_retry_in(35);
    state.last_error = Some("forecast request failed".to_string());
    insta::assert_snapshot!(
        "80x24_stale_retry_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn snapshot_80x24_offline_badge() {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle(61));
    state.refresh_meta.state = FreshnessState::Offline;
    state.refresh_meta.last_success = Some(Utc::now() - chrono::Duration::minutes(40));
    state.last_error = Some("forecast request failed".to_string());
    insta::assert_snapshot!(
        "80x24_offline_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn snapshot_80x24_syncing_badge() {
    let cli = cli();
    let mut state = ready_state_with_weather(&cli, fixture_bundle(61));
    state.fetch_in_flight = true;
    insta::assert_snapshot!(
        "80x24_syncing_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn regular_layout_renders_footer_shortcuts() {
    let rendered = render_to_string(120, 40, 0);
    assert!(rendered.contains("r Refresh"));
    assert!(rendered.contains("F1/? Help"));
}

macro_rules! assert_hourly_mode_snapshot {
    ($fn_name:ident, $snapshot:literal, $width:expr, $height:expr, $mode:expr) => {
        #[test]
        fn $fn_name() {
            insta::assert_snapshot!(
                $snapshot,
                render_with_hourly_mode_to_string($width, $height, 61, $mode)
            );
        }
    };
}

assert_hourly_mode_snapshot!(
    snapshot_120x40_hybrid,
    "120x40_hybrid",
    120,
    40,
    HourlyViewMode::Hybrid
);
assert_hourly_mode_snapshot!(
    snapshot_100x30_hybrid,
    "100x30_hybrid",
    100,
    30,
    HourlyViewMode::Hybrid
);
assert_hourly_mode_snapshot!(
    snapshot_80x24_hybrid,
    "80x24_hybrid",
    80,
    24,
    HourlyViewMode::Hybrid
);
assert_hourly_mode_snapshot!(
    snapshot_60x20_hybrid,
    "60x20_hybrid",
    60,
    20,
    HourlyViewMode::Hybrid
);
assert_hourly_mode_snapshot!(
    snapshot_120x40_chart,
    "120x40_chart",
    120,
    40,
    HourlyViewMode::Chart
);
assert_hourly_mode_snapshot!(
    snapshot_100x30_chart,
    "100x30_chart",
    100,
    30,
    HourlyViewMode::Chart
);
assert_hourly_mode_snapshot!(
    snapshot_80x24_chart,
    "80x24_chart",
    80,
    24,
    HourlyViewMode::Chart
);
assert_hourly_mode_snapshot!(
    snapshot_60x20_chart,
    "60x20_chart",
    60,
    20,
    HourlyViewMode::Chart
);

#[test]
fn narrow_layout_forces_table_for_hybrid_mode() {
    let rendered = render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Hybrid);
    assert!(rendered.contains("Hourly · Table"));
}

#[test]
fn narrow_layout_forces_table_for_chart_mode() {
    let rendered = render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Chart);
    assert!(rendered.contains("Hourly · Table"));
}

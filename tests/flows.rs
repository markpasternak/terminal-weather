#![allow(clippy::cast_precision_loss)]

mod common;

use common::{
    FixtureProfile, fixture_bundle as shared_fixture_bundle, state_with_weather, stockholm_cli,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::Ordering;
use terminal_weather::{
    app::{
        events::AppEvent,
        state::{AppState, SettingsSelection},
    },
    cli::{Cli, HourlyViewArg},
    domain::weather::{HourlyViewMode, Units},
    ui::layout::visible_hour_count,
};
use tokio::sync::mpsc;

fn flow_cli() -> Cli {
    stockholm_cli()
}

fn flow_bundle() -> terminal_weather::domain::weather::ForecastBundle {
    shared_fixture_bundle(FixtureProfile::Flow, 61)
}

struct FlowHarness {
    cli: Cli,
    state: AppState,
    tx: mpsc::Sender<AppEvent>,
    rx: mpsc::Receiver<AppEvent>,
}

impl FlowHarness {
    fn new(state: AppState, cli: Cli) -> Self {
        let (tx, rx) = mpsc::channel(8);
        Self { cli, state, tx, rx }
    }

    fn with_weather() -> Self {
        let cli = flow_cli();
        let state = state_with_weather(&cli, flow_bundle());
        Self::new(state, cli)
    }

    fn fresh() -> Self {
        let cli = flow_cli();
        let state = AppState::new(&cli);
        Self::new(state, cli)
    }

    async fn input(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let event = AppEvent::Input(Event::Key(KeyEvent::new(code, modifiers)));
        self.state
            .handle_event(event, &self.tx, &self.cli)
            .await
            .expect("input event should be handled");
    }

    async fn key(&mut self, code: KeyCode) {
        self.input(code, KeyModifiers::NONE).await;
    }

    async fn recv(&mut self) -> AppEvent {
        self.rx.recv().await.expect("event expected")
    }
}

#[tokio::test]
async fn flow_unit_toggle_changes_display_units() {
    let mut harness = FlowHarness::with_weather();

    harness.key(KeyCode::Char('f')).await;
    assert_eq!(harness.state.units, Units::Fahrenheit);

    harness.key(KeyCode::Char('c')).await;
    assert_eq!(harness.state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_hourly_scroll_clamps_bounds() {
    let mut harness = FlowHarness::with_weather();

    for _ in 0..50 {
        harness.key(KeyCode::Right).await;
    }

    let expected_max = harness.state.weather.as_ref().map_or(0, |bundle| {
        bundle
            .hourly
            .len()
            .saturating_sub(visible_hour_count(harness.state.viewport_width))
    });
    assert!(harness.state.hourly_offset <= expected_max);

    for _ in 0..50 {
        harness.key(KeyCode::Left).await;
    }

    assert_eq!(harness.state.hourly_offset, 0);
}

#[tokio::test]
async fn flow_ctrl_c_quits_without_toggling_units() {
    let mut harness = FlowHarness::with_weather();

    harness
        .input(KeyCode::Char('c'), KeyModifiers::CONTROL)
        .await;

    let event = harness.recv().await;
    assert!(matches!(event, AppEvent::Quit));
    assert_eq!(harness.state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_city_picker_keeps_dialog_open_while_typing_l() {
    let mut harness = FlowHarness::fresh();
    harness.state.city_picker_open = true;

    harness.key(KeyCode::Char('l')).await;

    assert!(harness.state.city_picker_open);
    assert_eq!(harness.state.city_query, "l");
}

#[tokio::test]
async fn flow_question_mark_opens_help_overlay() {
    let mut harness = FlowHarness::fresh();

    harness.key(KeyCode::Char('?')).await;

    assert!(harness.state.help_open);
}

#[tokio::test]
async fn flow_refresh_interval_setting_updates_runtime_value_immediately() {
    let mut harness = FlowHarness::fresh();
    harness.state.settings_open = true;
    harness.state.settings_selected = SettingsSelection::RefreshInterval;
    let previous = harness.state.settings.refresh_interval_secs;

    harness.key(KeyCode::Right).await;

    assert_ne!(harness.state.settings.refresh_interval_secs, previous);
    assert_eq!(
        harness
            .state
            .refresh_interval_secs_runtime
            .load(Ordering::Relaxed),
        harness.state.settings.refresh_interval_secs
    );
}

#[tokio::test]
async fn flow_f1_toggles_help_overlay() {
    let mut harness = FlowHarness::fresh();

    harness.key(KeyCode::F(1)).await;
    assert!(harness.state.help_open);

    harness.key(KeyCode::F(1)).await;
    assert!(!harness.state.help_open);
}

#[tokio::test]
async fn flow_esc_closes_help_before_quit() {
    let mut harness = FlowHarness::fresh();
    harness.state.help_open = true;

    harness.key(KeyCode::Esc).await;

    assert!(!harness.state.help_open);
    assert!(harness.rx.try_recv().is_err());
}

#[tokio::test]
async fn flow_ctrl_l_requests_force_redraw() {
    let mut harness = FlowHarness::fresh();

    harness
        .input(KeyCode::Char('l'), KeyModifiers::CONTROL)
        .await;

    let event = harness.recv().await;
    assert!(matches!(event, AppEvent::ForceRedraw));
}

#[tokio::test]
async fn flow_help_shortcut_ignored_while_city_picker_typing() {
    let mut harness = FlowHarness::fresh();
    harness.state.city_picker_open = true;

    harness.key(KeyCode::Char('?')).await;

    assert!(!harness.state.help_open);
    assert!(harness.state.city_picker_open);
}

#[tokio::test]
async fn flow_v_cycles_hourly_view_and_wraps() {
    let mut harness = FlowHarness::fresh();
    assert_eq!(harness.state.hourly_view_mode, HourlyViewMode::Table);

    for expected in [
        HourlyViewMode::Hybrid,
        HourlyViewMode::Chart,
        HourlyViewMode::Table,
    ] {
        harness.key(KeyCode::Char('v')).await;
        assert_eq!(harness.state.hourly_view_mode, expected);
    }
}

#[tokio::test]
async fn flow_settings_hourly_view_updates_runtime_mode() {
    let mut harness = FlowHarness::fresh();
    harness.state.settings_open = true;
    for _ in 0..5 {
        harness.key(KeyCode::Down).await;
    }

    harness.key(KeyCode::Right).await;

    assert_eq!(harness.state.hourly_view_mode, HourlyViewMode::Hybrid);
    assert_eq!(harness.state.settings.hourly_view, HourlyViewMode::Hybrid);
}

#[test]
fn flow_cli_hourly_view_override_applies_at_runtime() {
    let mut override_cli = flow_cli();
    override_cli.hourly_view = Some(HourlyViewArg::Chart);
    let state = AppState::new(&override_cli);
    assert_eq!(state.hourly_view_mode, HourlyViewMode::Chart);
    assert_eq!(state.settings.hourly_view, HourlyViewMode::Table);
}

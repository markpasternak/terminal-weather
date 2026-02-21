#![allow(clippy::cast_precision_loss)]

mod common;

use common::{
    FixtureProfile, assert_fixture_bundle_shape, assert_stockholm_cli_shape,
    fixture_bundle as shared_fixture_bundle, state_with_weather, stockholm_cli,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::Ordering;
use terminal_weather::{
    app::{
        events::AppEvent,
        state::{AppState, SettingsSelection},
    },
    cli::HourlyViewArg,
    domain::weather::{HourlyViewMode, Units},
    ui::layout::visible_hour_count,
};
use tokio::sync::mpsc;

fn cli() -> terminal_weather::cli::Cli {
    stockholm_cli()
}

fn fixture_bundle() -> terminal_weather::domain::weather::ForecastBundle {
    shared_fixture_bundle(FixtureProfile::Flow, 61)
}

fn input_key(code: KeyCode, modifiers: KeyModifiers) -> AppEvent {
    AppEvent::Input(Event::Key(KeyEvent::new(code, modifiers)))
}

async fn send_input(
    state: &mut AppState,
    tx: &mpsc::Sender<AppEvent>,
    cli: &terminal_weather::cli::Cli,
    code: KeyCode,
    modifiers: KeyModifiers,
) {
    state
        .handle_event(input_key(code, modifiers), tx, cli)
        .await
        .expect("input event should be handled");
}

async fn send_key(
    state: &mut AppState,
    tx: &mpsc::Sender<AppEvent>,
    cli: &terminal_weather::cli::Cli,
    code: KeyCode,
) {
    send_input(state, tx, cli, code, KeyModifiers::NONE).await;
}

#[tokio::test]
async fn flow_unit_toggle_changes_display_units() {
    let cli = cli();
    let mut state = state_with_weather(&cli, fixture_bundle());
    let (tx, _rx) = mpsc::channel(8);

    send_key(&mut state, &tx, &cli, KeyCode::Char('f')).await;
    assert_eq!(state.units, Units::Fahrenheit);

    send_key(&mut state, &tx, &cli, KeyCode::Char('c')).await;
    assert_eq!(state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_hourly_scroll_clamps_bounds() {
    let cli = cli();
    let mut state = state_with_weather(&cli, fixture_bundle());
    let (tx, _rx) = mpsc::channel(8);

    for _ in 0..50 {
        send_key(&mut state, &tx, &cli, KeyCode::Right).await;
    }

    let expected_max = state.weather.as_ref().map_or(0, |bundle| {
        bundle
            .hourly
            .len()
            .saturating_sub(visible_hour_count(state.viewport_width))
    });
    assert!(state.hourly_offset <= expected_max);

    for _ in 0..50 {
        send_key(&mut state, &tx, &cli, KeyCode::Left).await;
    }

    assert_eq!(state.hourly_offset, 0);
}

#[tokio::test]
async fn flow_ctrl_c_quits_without_toggling_units() {
    let cli = cli();
    let mut state = state_with_weather(&cli, fixture_bundle());
    let (tx, mut rx) = mpsc::channel(8);

    send_input(
        &mut state,
        &tx,
        &cli,
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )
    .await;

    let event = rx.recv().await.expect("quit event");
    assert!(matches!(event, AppEvent::Quit));
    assert_eq!(state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_city_picker_keeps_dialog_open_while_typing_l() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    state.city_picker_open = true;

    send_key(&mut state, &tx, &cli, KeyCode::Char('l')).await;

    assert!(state.city_picker_open);
    assert_eq!(state.city_query, "l");
}

#[tokio::test]
async fn flow_question_mark_opens_help_overlay() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);

    send_key(&mut state, &tx, &cli, KeyCode::Char('?')).await;

    assert!(state.help_open);
}

#[tokio::test]
async fn flow_refresh_interval_setting_updates_runtime_value_immediately() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    state.settings_open = true;
    state.settings_selected = SettingsSelection::RefreshInterval;
    let previous = state.settings.refresh_interval_secs;

    send_key(&mut state, &tx, &cli, KeyCode::Right).await;

    assert_ne!(state.settings.refresh_interval_secs, previous);
    assert_eq!(
        state.refresh_interval_secs_runtime.load(Ordering::Relaxed),
        state.settings.refresh_interval_secs
    );
}

#[tokio::test]
async fn flow_f1_toggles_help_overlay() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);

    send_key(&mut state, &tx, &cli, KeyCode::F(1)).await;
    assert!(state.help_open);

    send_key(&mut state, &tx, &cli, KeyCode::F(1)).await;
    assert!(!state.help_open);
}

#[tokio::test]
async fn flow_esc_closes_help_before_quit() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, mut rx) = mpsc::channel(8);
    state.help_open = true;

    send_key(&mut state, &tx, &cli, KeyCode::Esc).await;

    assert!(!state.help_open);
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn flow_ctrl_l_requests_force_redraw() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, mut rx) = mpsc::channel(8);

    send_input(
        &mut state,
        &tx,
        &cli,
        KeyCode::Char('l'),
        KeyModifiers::CONTROL,
    )
    .await;

    let event = rx.recv().await.expect("event expected");
    assert!(matches!(event, AppEvent::ForceRedraw));
}

#[tokio::test]
async fn flow_help_shortcut_ignored_while_city_picker_typing() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    state.city_picker_open = true;

    send_key(&mut state, &tx, &cli, KeyCode::Char('?')).await;

    assert!(!state.help_open);
    assert!(state.city_picker_open);
}

#[tokio::test]
async fn flow_v_cycles_hourly_view_and_wraps() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    assert_eq!(state.hourly_view_mode, HourlyViewMode::Table);

    for expected in [
        HourlyViewMode::Hybrid,
        HourlyViewMode::Chart,
        HourlyViewMode::Table,
    ] {
        send_key(&mut state, &tx, &cli, KeyCode::Char('v')).await;
        assert_eq!(state.hourly_view_mode, expected);
    }
}

#[tokio::test]
async fn flow_settings_hourly_view_updates_runtime_mode() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    state.settings_open = true;
    for _ in 0..5 {
        send_key(&mut state, &tx, &cli, KeyCode::Down).await;
    }

    send_key(&mut state, &tx, &cli, KeyCode::Right).await;

    assert_eq!(state.hourly_view_mode, HourlyViewMode::Hybrid);
    assert_eq!(state.settings.hourly_view, HourlyViewMode::Hybrid);
}

#[test]
fn flow_cli_hourly_view_override_applies_at_runtime() {
    let mut override_cli = cli();
    override_cli.hourly_view = Some(HourlyViewArg::Chart);
    let state = AppState::new(&override_cli);
    assert_eq!(state.hourly_view_mode, HourlyViewMode::Chart);
    assert_eq!(state.settings.hourly_view, HourlyViewMode::Table);
}

#[test]
fn fixture_bundle_shape_contract() {
    let bundle = fixture_bundle();
    assert_fixture_bundle_shape(&bundle, 24, 7, 61);
}

#[test]
fn cli_shape_contract_for_flow_tests() {
    let cli = cli();
    assert_stockholm_cli_shape(&cli);
}

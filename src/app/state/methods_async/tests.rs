use super::*;
use chrono::Duration;
use tokio::sync::mpsc;

fn state() -> AppState {
    AppState::new(&crate::test_support::state_test_cli())
}

fn weather_state_with_hours(count: usize) -> AppState {
    let mut state = state();
    let mut bundle = crate::test_support::sample_bundle();
    let base = bundle.hourly[0].clone();
    let hourly = (0..count)
        .map(|idx| {
            let mut item = base.clone();
            item.time = base.time + Duration::hours(i64::try_from(idx).unwrap_or(0));
            item
        })
        .collect::<Vec<_>>();
    bundle.hourly = hourly;
    state.weather = Some(bundle);
    state
}

#[test]
fn command_from_char_maps_known_commands() {
    assert_eq!(command_from_char('q'), Some(KeyCommand::Quit));
    assert_eq!(command_from_char('s'), Some(KeyCommand::OpenSettings));
    assert_eq!(command_from_char('l'), Some(KeyCommand::OpenCityPicker));
    assert_eq!(command_from_char('r'), Some(KeyCommand::Refresh));
    assert_eq!(command_from_char('f'), Some(KeyCommand::SetFahrenheit));
    assert_eq!(command_from_char('c'), Some(KeyCommand::SetCelsius));
    assert_eq!(command_from_char('v'), Some(KeyCommand::CycleHourlyView));
    assert_eq!(command_from_char('x'), None);
}

#[tokio::test]
async fn control_shortcuts_emit_expected_events() {
    let state = state();
    let (tx, mut rx) = mpsc::channel(4);

    assert!(
        state
            .handle_control_shortcuts(
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                &tx
            )
            .await
            .expect("ctrl+c")
    );
    assert!(matches!(rx.recv().await, Some(AppEvent::Quit)));

    assert!(
        state
            .handle_control_shortcuts(
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
                &tx
            )
            .await
            .expect("ctrl+l")
    );
    assert!(matches!(rx.recv().await, Some(AppEvent::ForceRedraw)));
}

#[tokio::test]
async fn global_main_key_handles_help_and_escape() {
    let mut state = state();
    let (tx, mut rx) = mpsc::channel(4);

    assert!(
        state
            .handle_global_main_key(KeyCode::F(1), &tx)
            .await
            .expect("f1")
    );
    assert!(state.help_open);

    assert!(
        state
            .handle_global_main_key(KeyCode::Esc, &tx)
            .await
            .expect("esc")
    );
    assert!(matches!(rx.recv().await, Some(AppEvent::Quit)));
}

#[test]
fn hourly_navigation_moves_and_clamps_bounds() {
    let mut state = weather_state_with_hours(24);
    state.viewport_width = 80;

    for _ in 0..100 {
        let _ = state.handle_hourly_navigation_key(KeyCode::Right);
    }
    let max = state
        .weather
        .as_ref()
        .map_or(0, |w| w.hourly.len().saturating_sub(1));
    assert!(state.hourly_cursor <= max);
    assert!(state.hourly_offset <= state.hourly_cursor);

    for _ in 0..100 {
        let _ = state.handle_hourly_navigation_key(KeyCode::Left);
    }
    assert_eq!(state.hourly_cursor, 0);
    assert_eq!(state.hourly_offset, 0);
}

#[test]
fn open_helpers_toggle_expected_modal_state() {
    let mut state = state();
    state.open_help_overlay();
    assert!(state.help_open);
    assert!(!state.settings_open);
    assert!(!state.city_picker_open);

    state.open_settings_panel();
    assert!(state.settings_open);
    assert!(!state.help_open);
    assert!(!state.city_picker_open);

    state.open_city_picker();
    assert!(state.city_picker_open);
    assert!(!state.settings_open);
    assert!(!state.help_open);
    assert_eq!(state.city_history_selected, 0);
}

#[tokio::test]
async fn handle_char_command_cycles_hourly_view() {
    let mut state = state();
    let (tx, _rx) = mpsc::channel(2);
    let cli = crate::test_support::state_test_cli();
    assert_eq!(state.hourly_view_mode, HourlyViewMode::Table);
    state
        .handle_char_command(
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE),
            &tx,
            &cli,
        )
        .await
        .expect("v command");
    assert_eq!(state.hourly_view_mode, HourlyViewMode::Hybrid);
}

#[test]
fn try_select_pending_location_requires_selecting_mode() {
    let mut state = state();
    let location = Location::from_coords(59.3, 18.0);
    state.pending_locations = vec![location];
    state.mode = AppMode::Ready;
    let (tx, _rx) = mpsc::channel(2);
    assert!(!state.try_select_pending_location(KeyCode::Char('1'), &tx));
    assert_eq!(state.pending_locations.len(), 1);
}

#[tokio::test]
async fn try_select_pending_location_consumes_selected_entry() {
    let mut state = state();
    state.forecast_url_override = Some("http://127.0.0.1:1".to_string());
    state.air_quality_url_override = Some("http://127.0.0.1:1".to_string());
    let mut location = Location::from_coords(59.3, 18.0);
    location.name = "Stockholm".to_string();
    state.pending_locations = vec![location];
    state.mode = AppMode::SelectingLocation;
    let (tx, _rx) = mpsc::channel(2);
    assert!(state.try_select_pending_location(KeyCode::Char('1'), &tx));
    assert_eq!(state.mode, AppMode::Loading);
    assert!(state.pending_locations.is_empty());
    assert_eq!(
        state
            .selected_location
            .as_ref()
            .map(|loc| loc.name.as_str()),
        Some("Stockholm")
    );
}

#[test]
fn sync_handlers_update_mode_and_flags() {
    let mut state = state();
    let (tx, _rx) = mpsc::channel(2);

    state.handle_sync_event(AppEvent::FetchStarted, &tx);
    assert!(state.fetch_in_flight);

    state.handle_sync_event(
        AppEvent::GeocodeResolved(GeocodeResolution::NotFound("Missing".to_string())),
        &tx,
    );
    assert_eq!(state.mode, AppMode::Error);
    assert!(
        state
            .last_error
            .as_deref()
            .is_some_and(|msg| msg.contains("Missing"))
    );

    state.handle_sync_event(AppEvent::Quit, &tx);
    assert_eq!(state.mode, AppMode::Quit);
}

#[test]
fn fetch_succeeded_resets_state_and_caches_bundle() {
    let mut state = state();
    let bundle = crate::test_support::sample_bundle();
    state.handle_fetch_succeeded(bundle.clone());

    assert_eq!(state.mode, AppMode::Ready);
    assert!(!state.fetch_in_flight);
    assert!(state.weather.is_some());
    assert_eq!(state.hourly_offset, 0);
    assert_eq!(state.hourly_cursor, 0);
    assert_eq!(state.refresh_meta.consecutive_failures, 0);
}

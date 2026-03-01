use super::*;
use crate::cli::{HeroVisualArg, ThemeArg};
use serde_json::Value;
use tokio::sync::mpsc;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const TEST_LAT: f64 = 59.3293;
const TEST_LON: f64 = 18.0686;

fn test_state() -> AppState {
    AppState::new(&crate::test_support::state_test_cli())
}

fn coords_cli() -> Cli {
    let mut cli = crate::test_support::state_test_cli();
    cli.lat = Some(TEST_LAT);
    cli.lon = Some(TEST_LON);
    cli
}

fn stockholm_reverse_payload() -> Value {
    serde_json::json!({
        "address": {
            "city": "Stockholm",
            "state": "Stockholm County",
            "country": "Sweden"
        }
    })
}

async fn reverse_geocoder_client(
    status: u16,
    payload: Option<Value>,
) -> (MockServer, GeocodeClient) {
    let server = MockServer::start().await;
    let template = payload.map_or_else(
        || ResponseTemplate::new(status),
        |body| ResponseTemplate::new(status).set_body_json(body),
    );
    Mock::given(method("GET"))
        .and(path("/v1/reverse"))
        .respond_with(template)
        .mount(&server)
        .await;
    let geocoder = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));
    (server, geocoder)
}

#[test]
fn fetch_blocked_when_in_flight_or_selecting_location() {
    let mut state = test_state();
    state.fetch_in_flight = true;
    assert!(state.fetch_blocked());

    state.fetch_in_flight = false;
    state.mode = AppMode::SelectingLocation;
    assert!(state.fetch_blocked());

    state.mode = AppMode::Ready;
    assert!(!state.fetch_blocked());
}

#[test]
fn should_auto_lookup_only_without_city_or_lat() {
    let mut cli = crate::test_support::state_test_cli();
    assert!(AppState::should_auto_lookup(&cli));

    cli.city = Some("Berlin".to_string());
    assert!(!AppState::should_auto_lookup(&cli));

    let mut cli = crate::test_support::state_test_cli();
    cli.lat = Some(59.3);
    assert!(!AppState::should_auto_lookup(&cli));
}

#[tokio::test]
async fn try_fetch_coords_emits_selected_location_when_present() {
    let cli = coords_cli();
    let (_server, geocoder) = reverse_geocoder_client(200, Some(stockholm_reverse_payload())).await;

    let (tx, mut rx) = mpsc::channel(2);
    let handled = AppState::try_fetch_coords_with_geocoder(&tx, &cli, &geocoder)
        .await
        .expect("coords should be handled");

    assert!(handled);
    let event = rx.recv().await.expect("event");
    assert!(matches!(
        event,
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(location))
        if location.name == "Stockholm"
        && (location.latitude - TEST_LAT).abs() < f64::EPSILON
    ));
}

#[tokio::test]
async fn try_fetch_coords_falls_back_when_reverse_geocode_fails() {
    let cli = coords_cli();
    let (_server, geocoder) = reverse_geocoder_client(500, None).await;

    let (tx, mut rx) = mpsc::channel(2);
    let handled = AppState::try_fetch_coords_with_geocoder(&tx, &cli, &geocoder)
        .await
        .expect("coords should be handled");

    assert!(handled);
    let event = rx.recv().await.expect("event");
    assert!(matches!(
        event,
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(location))
        if location.name == "59.3293, 18.0686"
    ));
}

#[tokio::test]
async fn resolve_saved_location_updates_coordinate_label_when_reverse_available() {
    let (_server, geocoder) = reverse_geocoder_client(200, Some(stockholm_reverse_payload())).await;

    let location = resolve_saved_location_with_reverse_geocoder(
        &geocoder,
        Location::from_coords(TEST_LAT, TEST_LON),
    )
    .await;

    assert_eq!(location.name, "Stockholm");
}

#[tokio::test]
async fn resolve_saved_location_keeps_named_location_without_reverse_lookup() {
    let geocoder = GeocodeClient::with_base_url("http://127.0.0.1:9");
    let named = crate::test_support::stockholm_location();

    let location = resolve_saved_location_with_reverse_geocoder(&geocoder, named.clone()).await;

    assert_eq!(location.name, named.name);
    assert_eq!(location.country, named.country);
}

#[tokio::test]
async fn try_fetch_coords_returns_false_without_complete_pair() {
    let mut cli = coords_cli();
    cli.lon = None;
    let (tx, mut rx) = mpsc::channel(2);
    let handled = AppState::try_fetch_coords(&tx, &cli)
        .await
        .expect("coords path should succeed");
    assert!(!handled);
    assert!(rx.try_recv().is_err());
}

#[test]
fn build_forecast_client_honors_override_combinations() {
    let mut state = test_state();
    state.forecast_url_override = Some("https://example.test/forecast".to_string());
    state.air_quality_url_override = Some("https://example.test/aq".to_string());
    let both = format!("{:?}", state.build_forecast_client());
    assert!(both.contains("https://example.test/forecast"));
    assert!(both.contains("https://example.test/aq"));

    state.air_quality_url_override = None;
    let forecast_only = format!("{:?}", state.build_forecast_client());
    assert!(forecast_only.contains("https://example.test/forecast"));

    state.forecast_url_override = None;
    state.air_quality_url_override = Some("https://example.test/aq2".to_string());
    let aq_only = format!("{:?}", state.build_forecast_client());
    assert!(aq_only.contains("https://example.test/aq2"));
}

#[tokio::test]
async fn handle_demo_action_quit_emits_quit_event() {
    let mut state = test_state();
    let (tx, mut rx) = mpsc::channel(2);
    state
        .handle_demo_action(DemoAction::Quit, &tx)
        .await
        .expect("quit should be handled");
    assert!(matches!(rx.recv().await, Some(AppEvent::Quit)));
}

#[test]
fn demo_open_city_picker_sets_expected_ui_state() {
    let mut state = test_state();
    state.settings_open = true;
    state.demo_open_city_picker("Tokyo");
    assert!(!state.settings_open);
    assert!(state.city_picker_open);
    assert_eq!(state.city_query, "Tokyo");
    assert_eq!(state.city_history_selected, 0);
}

#[test]
fn demo_set_hero_visual_changes_only_when_needed() {
    let mut state = test_state();
    state.settings.hero_visual = HeroVisualArg::AtmosCanvas;
    state.demo_set_hero_visual(HeroVisualArg::GaugeCluster);
    assert_eq!(state.settings.hero_visual, HeroVisualArg::GaugeCluster);
    let prev = state.settings.hero_visual;
    state.demo_set_hero_visual(HeroVisualArg::GaugeCluster);
    assert_eq!(state.settings.hero_visual, prev);
}

#[test]
fn demo_set_theme_changes_only_when_needed() {
    let mut state = test_state();
    state.settings.theme = ThemeArg::Auto;
    state.demo_set_theme(ThemeArg::Nord);
    assert_eq!(state.settings.theme, ThemeArg::Nord);
    let prev = state.settings.theme;
    state.demo_set_theme(ThemeArg::Nord);
    assert_eq!(state.settings.theme, prev);
}

#[tokio::test]
async fn try_fetch_existing_location_returns_false_without_selection() {
    let state = test_state();
    let (tx, _rx) = mpsc::channel(2);
    assert!(!state.try_fetch_existing_location(&tx).await);
}

#[tokio::test]
async fn try_fetch_existing_location_returns_true_with_selection() {
    let mut state = test_state();
    state.selected_location = Some(crate::test_support::stockholm_location());
    state.forecast_url_override = Some("http://127.0.0.1:1".to_string());
    state.air_quality_url_override = Some("http://127.0.0.1:1".to_string());
    let (tx, _rx) = mpsc::channel(2);
    assert!(state.try_fetch_existing_location(&tx).await);
}

#[tokio::test]
async fn start_fetch_returns_early_when_blocked() {
    let mut state = test_state();
    state.fetch_in_flight = true;
    let cli = crate::test_support::state_test_cli();
    let (tx, mut rx) = mpsc::channel(4);

    state
        .start_fetch(&tx, &cli)
        .await
        .expect("blocked start_fetch should not fail");

    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn start_fetch_auto_lookup_sets_detecting_loading_message() {
    let mut state = test_state();
    let cli = crate::test_support::state_test_cli();
    let (tx, mut rx) = mpsc::channel(4);

    state
        .start_fetch(&tx, &cli)
        .await
        .expect("auto lookup start_fetch should not fail");

    assert!(matches!(rx.recv().await, Some(AppEvent::FetchStarted)));
    assert_eq!(state.loading_message, "Detecting location...");
}

#[tokio::test]
async fn start_fetch_city_lookup_keeps_existing_loading_message() {
    let mut state = test_state();
    state.loading_message = "Initializing...".to_string();
    let mut cli = crate::test_support::state_test_cli();
    cli.city = Some("Berlin".to_string());
    let (tx, mut rx) = mpsc::channel(4);

    state
        .start_fetch(&tx, &cli)
        .await
        .expect("city lookup start_fetch should not fail");

    assert!(matches!(rx.recv().await, Some(AppEvent::FetchStarted)));
    assert_eq!(state.loading_message, "Initializing...");
}

#[tokio::test]
async fn apply_demo_action_handles_all_non_quit_variants() {
    let mut state = test_state();
    let (tx, _rx) = mpsc::channel(4);
    let location = crate::test_support::stockholm_location();

    state.apply_demo_action(DemoAction::OpenSettings, &tx);
    assert!(state.settings_open);
    assert_eq!(state.settings_selected, SettingsSelection::HeroVisual);

    state.apply_demo_action(DemoAction::SetHeroVisual(HeroVisualArg::GaugeCluster), &tx);
    assert_eq!(state.settings.hero_visual, HeroVisualArg::GaugeCluster);

    state.apply_demo_action(DemoAction::SetTheme(ThemeArg::Nord), &tx);
    assert_eq!(state.settings.theme, ThemeArg::Nord);

    state.apply_demo_action(DemoAction::OpenCityPicker("Tokyo".to_string()), &tx);
    assert!(state.city_picker_open);
    assert_eq!(state.city_query, "Tokyo");

    state.apply_demo_action(DemoAction::SwitchCity(location), &tx);
    assert!(!state.city_picker_open);
    assert!(state.city_status.is_some());

    state.settings_open = true;
    state.apply_demo_action(DemoAction::CloseSettings, &tx);
    assert!(!state.settings_open);
}

#[tokio::test]
async fn fetch_forecast_emits_failed_event_on_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let mut state = test_state();
    state.forecast_url_override = Some(server.uri());
    state.air_quality_url_override = Some(server.uri());

    let (tx, mut rx) = mpsc::channel(2);
    let location = Location::from_coords(TEST_LAT, TEST_LON);

    state.fetch_forecast(&tx, location);

    let event = rx.recv().await.expect("event should be sent");
    assert!(matches!(event, AppEvent::FetchFailed(_)));
}

use clap::Parser;
use terminal_weather::{
    app::{
        events::AppEvent,
        state::{AppMode, AppState},
    },
    cli::Cli,
    domain::weather::{ForecastBundle, GeocodeResolution, Location},
};
use tokio::sync::mpsc;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

#[tokio::test]
async fn caching_reduces_network_calls() {
    let server = setup_mock_server().await;
    let cli = Cli::parse_from(vec![
        "terminal-weather".to_string(),
        "--forecast-url".to_string(),
        server.uri(),
        "--air-quality-url".to_string(),
        server.uri(),
    ]);
    let mut app = AppState::new(&cli);
    let (tx, mut rx) = mpsc::channel(100);
    let loc_a = Location::from_coords(10.0, 10.0);
    let loc_b = Location::from_coords(20.0, 20.0);

    request_and_expect_fetch(
        &mut app,
        &cli,
        &tx,
        &mut rx,
        loc_a.clone(),
        10.0,
        "Loc A (1st)",
    )
    .await;
    request_and_expect_fetch(
        &mut app,
        &cli,
        &tx,
        &mut rx,
        loc_b.clone(),
        20.0,
        "Loc B (1st)",
    )
    .await;
    request_and_expect_ready_or_fetch(&mut app, &cli, &tx, &mut rx, loc_a, 10.0).await;
    request_and_expect_ready_or_fetch(&mut app, &cli, &tx, &mut rx, loc_b, 20.0).await;

    let requests = server.received_requests().await.unwrap();
    let request_count = requests.len();
    assert_eq!(
        request_count, 4,
        "Expected 4 requests with cache, got {request_count}"
    );
}

async fn setup_mock_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_forecast_payload()))
        .mount(&server)
        .await;
    server
}

fn mock_forecast_payload() -> serde_json::Value {
    serde_json::json!({
        "current": mock_current_payload(),
        "hourly": mock_hourly_payload(),
        "daily": mock_daily_payload(),
    })
}

fn mock_current_payload() -> serde_json::Value {
    serde_json::json!({
        "temperature_2m": 20.0,
        "relative_humidity_2m": 50.0,
        "apparent_temperature": 20.0,
        "dew_point_2m": 10.0,
        "weather_code": 0,
        "precipitation": 0.0,
        "cloud_cover": 0.0,
        "pressure_msl": 1013.0,
        "visibility": 10000.0,
        "wind_speed_10m": 5.0,
        "wind_gusts_10m": 10.0,
        "wind_direction_10m": 180.0,
        "is_day": 1
    })
}

fn mock_hourly_payload() -> serde_json::Value {
    serde_json::json!({
        "time": ["2024-01-01T00:00"],
        "temperature_2m": [20.0],
        "weather_code": [0],
        "is_day": [1],
        "relative_humidity_2m": [50.0],
        "precipitation_probability": [0.0],
        "precipitation": [0.0],
        "rain": [0.0],
        "snowfall": [0.0],
        "wind_speed_10m": [5.0],
        "wind_gusts_10m": [10.0],
        "pressure_msl": [1013.0],
        "visibility": [10000.0],
        "cloud_cover": [0.0],
        "cloud_cover_low": [0.0],
        "cloud_cover_mid": [0.0],
        "cloud_cover_high": [0.0]
    })
}

fn mock_daily_payload() -> serde_json::Value {
    serde_json::json!({
        "time": ["2024-01-01"],
        "weather_code": [0],
        "temperature_2m_max": [25.0],
        "temperature_2m_min": [15.0],
        "sunrise": ["2024-01-01T06:00"],
        "sunset": ["2024-01-01T18:00"],
        "uv_index_max": [5.0],
        "precipitation_probability_max": [0.0],
        "precipitation_sum": [0.0],
        "rain_sum": [0.0],
        "snowfall_sum": [0.0],
        "precipitation_hours": [0.0],
        "wind_gusts_10m_max": [10.0],
        "daylight_duration": [43200.0],
        "sunshine_duration": [43200.0]
    })
}

async fn request_and_expect_fetch(
    app: &mut AppState,
    cli: &Cli,
    tx: &mpsc::Sender<AppEvent>,
    rx: &mut mpsc::Receiver<AppEvent>,
    location: Location,
    expected_latitude: f64,
    label: &str,
) {
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(location)),
        tx,
        cli,
    )
    .await
    .unwrap();

    if app.mode == AppMode::Loading {
        let bundle = wait_for_success(rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), tx, cli)
            .await
            .unwrap();
        assert_latitude(bundle.location.latitude, expected_latitude);
    } else {
        panic!("Expected network fetch for {label}");
    }
}

async fn request_and_expect_ready_or_fetch(
    app: &mut AppState,
    cli: &Cli,
    tx: &mpsc::Sender<AppEvent>,
    rx: &mut mpsc::Receiver<AppEvent>,
    location: Location,
    expected_latitude: f64,
) {
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(location)),
        tx,
        cli,
    )
    .await
    .unwrap();

    if app.mode == AppMode::Ready {
        let cached_latitude = app.weather.as_ref().unwrap().location.latitude;
        assert_latitude(cached_latitude, expected_latitude);
    } else {
        let bundle = wait_for_success(rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), tx, cli)
            .await
            .unwrap();
        assert_latitude(bundle.location.latitude, expected_latitude);
    }
}

fn assert_latitude(actual: f64, expected: f64) {
    const EPSILON: f64 = 1e-6;
    let delta = (actual - expected).abs();
    assert!(
        delta <= EPSILON,
        "expected latitude {expected}, got {actual} (delta={delta})"
    );
}

async fn wait_for_success(rx: &mut mpsc::Receiver<AppEvent>) -> ForecastBundle {
    loop {
        match rx.recv().await {
            Some(AppEvent::FetchSucceeded(bundle)) => return bundle,
            Some(AppEvent::FetchFailed(err)) => panic!("Fetch failed: {err}"),
            Some(_) => {}
            None => panic!("Channel closed"),
        }
    }
}

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
    let server = MockServer::start().await;
    unsafe {
        std::env::set_var("TERMINAL_WEATHER_FORECAST_URL", server.uri());
        std::env::set_var("TERMINAL_WEATHER_AIR_QUALITY_URL", server.uri());
    }

    // Minimal valid JSON payload
    let payload = serde_json::json!({
        "current": {
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
        },
        "hourly": {
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
        },
        "daily": {
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
        }
    });

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .mount(&server)
        .await;

    let cli = Cli::parse_from(["terminal-weather"]);
    let mut app = AppState::new(&cli);
    let (tx, mut rx) = mpsc::channel(100);

    let loc_a = Location::from_coords(10.0, 10.0);
    let loc_b = Location::from_coords(20.0, 20.0);

    // 1. Request Loc A
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(loc_a.clone())),
        &tx,
        &cli,
    )
    .await
    .unwrap();

    if app.mode == AppMode::Loading {
        let bundle = wait_for_success(&mut rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), &tx, &cli)
            .await
            .unwrap();
        assert_eq!(bundle.location.latitude, 10.0);
    } else {
        panic!("Expected network fetch for Loc A (1st)");
    }

    // 2. Request Loc B
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(loc_b.clone())),
        &tx,
        &cli,
    )
    .await
    .unwrap();

    if app.mode == AppMode::Loading {
        let bundle = wait_for_success(&mut rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), &tx, &cli)
            .await
            .unwrap();
        assert_eq!(bundle.location.latitude, 20.0);
    } else {
        panic!("Expected network fetch for Loc B (1st)");
    }

    // 3. Request Loc A again (should be cached)
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(loc_a.clone())),
        &tx,
        &cli,
    )
    .await
    .unwrap();

    if app.mode == AppMode::Loading {
        // It might be stale, so it might trigger fetch. But payload timestamp is old?
        // In mock payload, timestamp is hardcoded "2024-01-01".
        // Current time is 2024+ (real time).
        // So it will be stale.
        // And "prevent redundant network calls" logic calls fetch if stale.
        // Wait, if it fetches, we get requests.
        // I want to verify cache behavior.
        // If stale, it updates synchronously from cache, THEN triggers fetch.
        // So `mode` will be `Ready` (from cache update), BUT `fetch_in_flight` will be true.
        // Wait, `switch_to_location` calls `handle_fetch_succeeded`. That sets `mode = Ready`.
        // Then if stale, it spawns task to send `FetchStarted`, and calls `fetch_forecast`.
        // So immediately after `handle_event` returns:
        // `mode` is `Ready`.
        // `fetch_in_flight` is NOT yet true (because `FetchStarted` is sent async, or `fetch_forecast` didn't set it yet).
        // Actually `fetch_forecast` spawns a task. It does NOT set `fetch_in_flight`.
        // `handle_fetch_started` sets it.

        // So `mode` should be `Ready`.
    }

    if app.mode != AppMode::Ready {
        let bundle = wait_for_success(&mut rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), &tx, &cli)
            .await
            .unwrap();
    } else {
        // Cache hit.
        assert_eq!(app.weather.as_ref().unwrap().location.latitude, 10.0);
    }

    // 4. Request Loc B again (should be cached)
    app.handle_event(
        AppEvent::GeocodeResolved(GeocodeResolution::Selected(loc_b.clone())),
        &tx,
        &cli,
    )
    .await
    .unwrap();

    if app.mode != AppMode::Ready {
        let bundle = wait_for_success(&mut rx).await;
        app.handle_event(AppEvent::FetchSucceeded(bundle.clone()), &tx, &cli)
            .await
            .unwrap();
    } else {
        // Cache hit.
        assert_eq!(app.weather.as_ref().unwrap().location.latitude, 20.0);
    }

    let requests = server.received_requests().await.unwrap();
    // Expect 2 fetches * 2 calls = 4.
    // If not cached, 4 fetches * 2 calls = 8.
    assert_eq!(
        requests.len(),
        4,
        "Expected 4 requests with cache, got {}",
        requests.len()
    );
}

async fn wait_for_success(rx: &mut mpsc::Receiver<AppEvent>) -> ForecastBundle {
    loop {
        match rx.recv().await {
            Some(AppEvent::FetchSucceeded(bundle)) => return bundle,
            Some(AppEvent::FetchFailed(err)) => panic!("Fetch failed: {}", err),
            Some(_) => {}
            None => panic!("Channel closed"),
        }
    }
}

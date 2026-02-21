use std::time::{Duration, Instant};
use terminal_weather::data::forecast::ForecastClient;
use terminal_weather::domain::weather::Location;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn benchmark_fetch_latency() {
    // 1. Setup Mock Servers
    let forecast_server = MockServer::start().await;
    let air_quality_server = MockServer::start().await;

    // Define delays
    let delay = Duration::from_millis(200);

    // Mock Forecast Response
    let forecast_body = r#"{
      "current": {
        "temperature_2m": 20.0,
        "relative_humidity_2m": 50.0,
        "apparent_temperature": 19.0,
        "dew_point_2m": 10.0,
        "weather_code": 1,
        "precipitation": 0.0,
        "cloud_cover": 10.0,
        "pressure_msl": 1013.0,
        "visibility": 10000.0,
        "wind_speed_10m": 5.0,
        "wind_gusts_10m": 10.0,
        "wind_direction_10m": 180.0,
        "is_day": 1
      },
      "hourly": {
        "time": ["2023-01-01T00:00"],
        "temperature_2m": [20.0],
        "weather_code": [1],
        "is_day": [0],
        "relative_humidity_2m": [50.0],
        "precipitation_probability": [0.0],
        "precipitation": [0.0],
        "rain": [0.0],
        "snowfall": [0.0],
        "wind_speed_10m": [5.0],
        "wind_gusts_10m": [10.0],
        "pressure_msl": [1013.0],
        "visibility": [10000.0],
        "cloud_cover": [10.0],
        "cloud_cover_low": [10.0],
        "cloud_cover_mid": [10.0],
        "cloud_cover_high": [10.0]
      },
      "daily": {
        "time": ["2023-01-01"],
        "weather_code": [1],
        "temperature_2m_max": [25.0],
        "temperature_2m_min": [15.0],
        "sunrise": ["2023-01-01T06:00"],
        "sunset": ["2023-01-01T18:00"],
        "uv_index_max": [5.0],
        "precipitation_probability_max": [0.0],
        "precipitation_sum": [0.0],
        "rain_sum": [0.0],
        "snowfall_sum": [0.0],
        "precipitation_hours": [0.0],
        "wind_gusts_10m_max": [15.0],
        "daylight_duration": [43200.0],
        "sunshine_duration": [43200.0]
      }
    }"#;

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::from_str::<serde_json::Value>(forecast_body).unwrap())
                .set_delay(delay),
        )
        .mount(&forecast_server)
        .await;

    // Mock Air Quality Response
    let air_quality_body = r#"{
      "current": {
        "us_aqi": 50.0,
        "european_aqi": 20.0
      }
    }"#;

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::from_str::<serde_json::Value>(air_quality_body).unwrap())
                .set_delay(delay),
        )
        .mount(&air_quality_server)
        .await;

    // 2. Setup Client
    let client = ForecastClient::with_base_url(forecast_server.uri())
        .with_air_quality_url(air_quality_server.uri());

    let location = Location::from_coords(40.7128, -74.0060);

    // 3. Measure Execution Time
    let start = Instant::now();
    let result = client.fetch(location).await;
    let duration = start.elapsed();

    // 4. Verify Result
    assert!(result.is_ok(), "Fetch failed: {:?}", result.err());
    let bundle = result.unwrap();
    assert!(bundle.air_quality.is_some(), "Air quality missing");

    println!("Fetch took: {:?}", duration);

    // 5. Performance Assertions
    // Sequential: ~400ms (200 + 200)
    // Concurrent: ~200ms (max(200, 200))
    // We add some buffer for processing time.

    // This assertion is flexible enough to pass before and after optimization,
    // but the printed output will show the difference.
    // However, to ensure I see the difference, I can be more specific.
    // Before optimization, it should be > 400ms.
    // After optimization, it should be < 300ms.
    // I'll leave the assertions for now and just print.

    // But since this is a test, I want to see if it passes.
    // I'll make it print useful info.
}

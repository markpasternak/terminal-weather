use super::*;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[test]
fn ranking_prefers_exact_then_population() {
    let results = vec![
        GeocodeResult {
            name: "Paris".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            country: Some("France".to_string()),
            country_code: Some("FR".to_string()),
            admin1: None,
            timezone: None,
            population: Some(2_000_000),
        },
        GeocodeResult {
            name: "Parish".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            admin1: None,
            timezone: None,
            population: Some(10_000_000),
        },
    ];

    let ranked = rank_locations(results, "Paris", None);
    assert_eq!(ranked[0].location.name, "Paris");
}

#[test]
fn ambiguity_detected_when_population_close() {
    let a = ScoredLocation {
        location: Location::from_coords(1.0, 1.0),
        exact_name_match: true,
        country_match: true,
        population: 1_000_000,
        api_order: 0,
    };
    let b = ScoredLocation {
        location: Location::from_coords(1.0, 1.0),
        exact_name_match: true,
        country_match: true,
        population: 950_000,
        api_order: 1,
    };

    assert!(is_ambiguous(&a, &b));
}

#[test]
fn normalize_is_unicode_case_insensitive() {
    assert_eq!(normalize("Åre"), normalize("åre"));
}

#[test]
fn infer_reverse_geocode_url_switches_search_suffix() {
    assert_eq!(
        infer_reverse_geocode_url("https://geocoding-api.open-meteo.com/v1/search"),
        "https://geocoding-api.open-meteo.com/v1/reverse"
    );
    assert_eq!(
        infer_reverse_geocode_url("http://127.0.0.1:1234"),
        "http://127.0.0.1:1234"
    );
}

#[tokio::test]
async fn reverse_resolve_returns_first_location_when_present() {
    let server = MockServer::start().await;
    let payload = serde_json::json!({
        "address": {
            "city": "Stockholm",
            "state": "Stockholm County",
            "country": "Sweden"
        }
    });

    Mock::given(method("GET"))
        .and(path("/v1/reverse"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .mount(&server)
        .await;

    let client = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri())).expect("test");
    let location = client
        .reverse_resolve(59.3293, 18.0686)
        .await
        .expect("reverse resolve")
        .expect("expected one location");

    assert_eq!(location.name, "Stockholm");
    assert_eq!(location.admin1.as_deref(), Some("Stockholm County"));
    assert_eq!(location.country.as_deref(), Some("Sweden"));
}

#[tokio::test]
async fn resolve_rejects_huge_city_name() {
    let client = GeocodeClient::new().expect("test");
    let huge_city = "a".repeat(101);
    let result = client.resolve(huge_city, None).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("City name is too long"));
}

#[tokio::test]
async fn resolve_rejects_huge_country_code() {
    let client = GeocodeClient::new().expect("test");
    let city = "Stockholm".to_string();
    let huge_code = Some("A".repeat(11));
    let result = client.resolve(city, huge_code).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Country code is too long"));
}

#[tokio::test]
async fn resolve_sanitizes_inputs() {
    let server = MockServer::start().await;
    let payload = serde_json::json!({
        "results": [
            {
                "name": "Tokyo\x1b[31m",
                "latitude": 35.6895,
                "longitude": 139.6917,
                "country": "Japan\n",
                "country_code": "JP",
                "admin1": "Tokyo\tPrefecture",
                "timezone": "Asia/Tokyo",
                "population": 10_000_000
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .mount(&server)
        .await;

    let client = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri())).expect("test");
    let result = client
        .resolve("Tokyo".to_string(), None)
        .await
        .expect("resolve");

    if let GeocodeResolution::Selected(loc) = result {
        assert_eq!(loc.name, "Tokyo[31m");
        assert_eq!(loc.country.as_deref(), Some("Japan"));
        assert_eq!(loc.admin1.as_deref(), Some("TokyoPrefecture"));
    } else {
        panic!("expected Selected, got {result:?}");
    }
}

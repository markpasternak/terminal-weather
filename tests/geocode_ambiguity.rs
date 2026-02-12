use terminal_weather::{data::geocode::GeocodeClient, domain::weather::GeocodeResolution};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

#[tokio::test]
async fn ambiguous_candidates_trigger_selector_resolution() {
    let server = MockServer::start().await;

    let body = r#"
    {
      "results": [
        {
          "name": "Springfield",
          "latitude": 39.799,
          "longitude": -89.644,
          "country": "United States",
          "country_code": "US",
          "admin1": "Illinois",
          "timezone": "America/Chicago",
          "population": 115000
        },
        {
          "name": "Springfield",
          "latitude": 44.046,
          "longitude": -123.022,
          "country": "United States",
          "country_code": "US",
          "admin1": "Oregon",
          "timezone": "America/Los_Angeles",
          "population": 112000
        }
      ]
    }
    "#;

    Mock::given(method("GET"))
        .and(path("/v1/search"))
        .and(query_param("name", "Springfield"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(&server)
        .await;

    let client = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));
    let result = client
        .resolve("Springfield".to_string(), Some("US".to_string()))
        .await
        .expect("resolve");

    match result {
        GeocodeResolution::NeedsDisambiguation(options) => assert_eq!(options.len(), 2),
        other => panic!("expected ambiguity, got {other:?}"),
    }
}

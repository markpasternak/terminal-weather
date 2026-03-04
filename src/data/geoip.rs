use crate::domain::weather::{Location, sanitize_text};
use reqwest::Client;
use serde::Deserialize;

const MAX_CITY_LEN: usize = 100;
const MAX_COUNTRY_LEN: usize = 100;
const MAX_REGION_LEN: usize = 100;
const MAX_TIMEZONE_LEN: usize = 50;

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    country_name: Option<String>,
    region: Option<String>,
    timezone: Option<String>,
}

pub async fn detect_location() -> Option<Location> {
    detect_location_with_url("https://ipapi.co/json/").await
}

async fn detect_location_with_url(url: &str) -> Option<Location> {
    let client = build_client()?;
    let response = fetch_response(&client, url).await?;
    response_to_location(response)
}

fn build_client() -> Option<Client> {
    Client::builder()
        .user_agent(concat!("terminal-weather/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()
}

async fn fetch_response(client: &Client, url: &str) -> Option<IpApiResponse> {
    let mut response = client.get(url).send().await.ok()?;
    let mut body_bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if body_bytes.len() + chunk.len() > 1024 * 1024 {
            return None;
        }
        body_bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body_bytes).ok()
}

fn response_to_location(response: IpApiResponse) -> Option<Location> {
    let name = valid_city(response.city)?;
    let (latitude, longitude) = valid_coordinates(response.latitude?, response.longitude?)?;
    Some(Location {
        name: sanitize_text(&name),
        latitude,
        longitude,
        country: sanitized_optional_field(response.country_name, MAX_COUNTRY_LEN),
        admin1: sanitized_optional_field(response.region, MAX_REGION_LEN),
        timezone: sanitized_optional_field(response.timezone, MAX_TIMEZONE_LEN),
        population: None,
    })
}

fn valid_city(city: Option<String>) -> Option<String> {
    city.filter(|value| !value.is_empty() && value.len() <= MAX_CITY_LEN)
}

fn valid_coordinates(latitude: f64, longitude: f64) -> Option<(f64, f64)> {
    if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) {
        Some((latitude, longitude))
    } else {
        None
    }
}

fn sanitized_optional_field(value: Option<String>, max_len: usize) -> Option<String> {
    value
        .filter(|entry| entry.len() <= max_len)
        .map(|entry| sanitize_text(&entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn detect_location_rejects_invalid_coordinates() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "city": "Stockholm",
            "latitude": 91.0,
            "longitude": 181.0,
            "country_name": "Sweden",
            "region": "Stockholm",
            "timezone": "Europe/Stockholm"
        });

        Mock::given(method("GET"))
            .and(path("/json/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let result = detect_location_with_url(&format!("{}/json/", server.uri())).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn detect_location_rejects_long_city() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "city": "A".repeat(MAX_CITY_LEN + 1),
            "latitude": 59.3293,
            "longitude": 18.0686,
            "country_name": "Sweden",
            "region": "Stockholm",
            "timezone": "Europe/Stockholm"
        });

        Mock::given(method("GET"))
            .and(path("/json/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let result = detect_location_with_url(&format!("{}/json/", server.uri())).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn detect_location_accepts_valid_city() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "city": "Stockholm",
            "latitude": 59.3293,
            "longitude": 18.0686,
            "country_name": "Sweden",
            "region": "Stockholm",
            "timezone": "Europe/Stockholm"
        });

        Mock::given(method("GET"))
            .and(path("/json/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let result = detect_location_with_url(&format!("{}/json/", server.uri())).await;
        let location = result.expect("should return location");
        assert_eq!(location.name, "Stockholm");
    }

    #[tokio::test]
    async fn detect_location_filters_long_fields() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "city": "Stockholm",
            "latitude": 59.3293,
            "longitude": 18.0686,
            "country_name": "A".repeat(MAX_COUNTRY_LEN + 1),
            "region": "B".repeat(MAX_REGION_LEN + 1),
            "timezone": "C".repeat(MAX_TIMEZONE_LEN + 1)
        });

        Mock::given(method("GET"))
            .and(path("/json/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let result = detect_location_with_url(&format!("{}/json/", server.uri())).await;
        let location = result.expect("should return location");

        // City is valid, so location is returned
        assert_eq!(location.name, "Stockholm");

        // Other fields should be None because they were filtered out
        assert!(location.country.is_none());
        assert!(location.admin1.is_none());
        assert!(location.timezone.is_none());
    }
}

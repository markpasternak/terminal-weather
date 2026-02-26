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
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let response: IpApiResponse = client.get(url).send().await.ok()?.json().await.ok()?;
    let name = response
        .city
        .filter(|c| !c.is_empty() && c.len() <= MAX_CITY_LEN)?;
    let latitude = response.latitude?;
    let longitude = response.longitude?;
    Some(Location {
        name: sanitize_text(&name),
        latitude,
        longitude,
        country: response
            .country_name
            .filter(|s| s.len() <= MAX_COUNTRY_LEN)
            .map(|s| sanitize_text(&s)),
        admin1: response
            .region
            .filter(|s| s.len() <= MAX_REGION_LEN)
            .map(|s| sanitize_text(&s)),
        timezone: response
            .timezone
            .filter(|s| s.len() <= MAX_TIMEZONE_LEN)
            .map(|s| sanitize_text(&s)),
        population: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

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

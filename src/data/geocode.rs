#![allow(clippy::cast_precision_loss, clippy::missing_errors_doc)]

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{GeocodeResolution, Location};

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const REVERSE_GEOCODE_URL: &str = "https://nominatim.openstreetmap.org/reverse";

#[derive(Debug, Clone)]
pub struct GeocodeClient {
    client: Client,
    base_url: String,
    reverse_url: String,
}

impl Default for GeocodeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GeocodeClient {
    #[must_use]
    pub fn new() -> Self {
        Self::with_urls(GEOCODE_URL, REVERSE_GEOCODE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let reverse_url = infer_reverse_geocode_url(&base_url);
        Self::with_urls(base_url, reverse_url)
    }

    pub fn with_urls(base_url: impl Into<String>, reverse_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent("terminal-weather/0.1")
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: base_url.into(),
            reverse_url: reverse_url.into(),
        }
    }

    pub async fn resolve(
        &self,
        city: String,
        country_code: Option<String>,
    ) -> Result<GeocodeResolution> {
        let mut request = self.client.get(&self.base_url).query(&[
            ("name", city.as_str()),
            ("count", "5"),
            ("language", "en"),
            ("format", "json"),
        ]);

        if let Some(code) = country_code.as_ref() {
            request = request.query(&[("countryCode", code)]);
        }

        let response = request
            .send()
            .await
            .context("geocoding request failed")?
            .error_for_status()
            .context("geocoding request returned non-success status")?;

        let payload: GeocodeResponse = response
            .json()
            .await
            .context("failed to decode geocoding response")?;

        if no_geocode_results(payload.results.as_ref()) {
            return Ok(GeocodeResolution::NotFound(city));
        }

        let mut ranked = rank_locations(
            payload.results.unwrap_or_default(),
            &city,
            country_code.as_deref(),
        );
        let top = ranked.remove(0);

        if let Some(options) = ambiguous_options(&top, &ranked) {
            return Ok(GeocodeResolution::NeedsDisambiguation(options));
        }

        Ok(GeocodeResolution::Selected(top.location))
    }

    pub async fn reverse_resolve(&self, latitude: f64, longitude: f64) -> Result<Option<Location>> {
        let response = self
            .client
            .get(&self.reverse_url)
            .query(&[
                ("lat", latitude.to_string()),
                ("lon", longitude.to_string()),
                ("accept-language", "en".to_string()),
                ("format", "jsonv2".to_string()),
            ])
            .send()
            .await
            .context("reverse geocoding request failed")?
            .error_for_status()
            .context("reverse geocoding request returned non-success status")?;

        let payload: ReverseGeocodeResponse = response
            .json()
            .await
            .context("failed to decode reverse geocoding response")?;

        Ok(payload
            .address
            .and_then(|address| location_from_reverse_address(address, latitude, longitude)))
    }
}

fn no_geocode_results(results: Option<&Vec<GeocodeResult>>) -> bool {
    results.is_none_or(Vec::is_empty)
}

fn ambiguous_options(top: &ScoredLocation, ranked: &[ScoredLocation]) -> Option<Vec<Location>> {
    let second = ranked.first()?;
    if !is_ambiguous(top, second) {
        return None;
    }
    let mut options = vec![top.location.clone()];
    options.extend(ranked.iter().map(|s| s.location.clone()).take(4));
    Some(options)
}

#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    results: Option<Vec<GeocodeResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodeResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    country_code: Option<String>,
    admin1: Option<String>,
    timezone: Option<String>,
    population: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ReverseGeocodeResponse {
    address: Option<ReverseAddress>,
}

#[derive(Debug, Deserialize)]
struct ReverseAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    county: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Clone)]
struct ScoredLocation {
    location: Location,
    exact_name_match: bool,
    country_match: bool,
    population: u64,
    api_order: usize,
}

fn geocode_result_to_location(entry: GeocodeResult) -> Location {
    Location {
        name: entry.name,
        latitude: entry.latitude,
        longitude: entry.longitude,
        country: entry.country,
        admin1: entry.admin1,
        timezone: entry.timezone,
        population: entry.population,
    }
}

fn infer_reverse_geocode_url(base_url: &str) -> String {
    base_url.strip_suffix("/search").map_or_else(
        || base_url.to_string(),
        |prefix| format!("{prefix}/reverse"),
    )
}

fn location_from_reverse_address(
    address: ReverseAddress,
    latitude: f64,
    longitude: f64,
) -> Option<Location> {
    let name = first_non_empty([
        address.city,
        address.town,
        address.village,
        address.municipality,
        address.county,
        address.state.clone(),
    ])?;
    Some(Location {
        name,
        latitude,
        longitude,
        country: address.country,
        admin1: address.state,
        timezone: None,
        population: None,
    })
}

fn first_non_empty(candidates: [Option<String>; 6]) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
}

fn rank_locations(
    results: Vec<GeocodeResult>,
    city: &str,
    country_code: Option<&str>,
) -> Vec<ScoredLocation> {
    let normalized_city = normalize(city);

    let mut scored: Vec<ScoredLocation> = results
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let exact_name_match = normalize(&entry.name) == normalized_city;
            let country_match = country_code.is_some_and(|cc| {
                entry
                    .country_code
                    .as_deref()
                    .is_some_and(|country| country.eq_ignore_ascii_case(cc))
            });
            let location = geocode_result_to_location(entry);
            let population = location.population.unwrap_or_default();

            ScoredLocation {
                location,
                exact_name_match,
                country_match,
                population,
                api_order: idx,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.exact_name_match
            .cmp(&a.exact_name_match)
            .then_with(|| b.country_match.cmp(&a.country_match))
            .then_with(|| b.population.cmp(&a.population))
            .then_with(|| a.api_order.cmp(&b.api_order))
    });

    scored
}

fn is_ambiguous(top: &ScoredLocation, second: &ScoredLocation) -> bool {
    if top.exact_name_match != second.exact_name_match {
        return false;
    }
    if top.country_match != second.country_match {
        return false;
    }

    let p1 = top.population.max(1) as f64;
    let p2 = second.population.max(1) as f64;
    let ratio = if p1 >= p2 { p1 / p2 } else { p2 / p1 };
    ratio <= 1.10
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
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

        let client = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));
        let location = client
            .reverse_resolve(59.3293, 18.0686)
            .await
            .expect("reverse resolve")
            .expect("expected one location");

        assert_eq!(location.name, "Stockholm");
        assert_eq!(location.admin1.as_deref(), Some("Stockholm County"));
        assert_eq!(location.country.as_deref(), Some("Sweden"));
    }
}

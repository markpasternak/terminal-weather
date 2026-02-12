use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{GeocodeResolution, Location};

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Clone)]
pub struct GeocodeClient {
    client: Client,
    base_url: String,
}

impl Default for GeocodeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GeocodeClient {
    pub fn new() -> Self {
        Self::with_base_url(GEOCODE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
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

        let Some(results) = payload.results else {
            return Ok(GeocodeResolution::NotFound(city));
        };

        if results.is_empty() {
            return Ok(GeocodeResolution::NotFound(city));
        }

        let mut ranked = rank_locations(results, &city, country_code.as_deref());
        let top = ranked.remove(0);

        if let Some(second) = ranked.first()
            && is_ambiguous(&top, second)
        {
            let mut options = vec![top.location.clone()];
            options.extend(ranked.into_iter().map(|s| s.location).take(4));
            return Ok(GeocodeResolution::NeedsDisambiguation(options));
        }

        Ok(GeocodeResolution::Selected(top.location))
    }
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

#[derive(Debug, Clone)]
struct ScoredLocation {
    location: Location,
    exact_name_match: bool,
    country_match: bool,
    population: u64,
    api_order: usize,
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
            let country_match = country_code
                .map(|cc| {
                    entry
                        .country_code
                        .as_deref()
                        .map(|country| country.eq_ignore_ascii_case(cc))
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            ScoredLocation {
                location: Location {
                    name: entry.name,
                    latitude: entry.latitude,
                    longitude: entry.longitude,
                    country: entry.country,
                    admin1: entry.admin1,
                    timezone: entry.timezone,
                    population: entry.population,
                },
                exact_name_match,
                country_match,
                population: entry.population.unwrap_or_default(),
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
        .to_ascii_lowercase()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

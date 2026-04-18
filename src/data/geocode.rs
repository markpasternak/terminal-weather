#![allow(clippy::cast_precision_loss, clippy::missing_errors_doc)]

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::data::http::apply_loopback_proxy_policy;
use crate::domain::weather::{GeocodeResolution, Location, sanitize_text};

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const REVERSE_GEOCODE_URL: &str = "https://nominatim.openstreetmap.org/reverse";

#[derive(Debug, Clone)]
pub struct GeocodeClient {
    client: Client,
    base_url: String,
    reverse_url: String,
}

impl GeocodeClient {
    pub fn new() -> Result<Self> {
        Self::with_urls(GEOCODE_URL, REVERSE_GEOCODE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let reverse_url = infer_reverse_geocode_url(&base_url);
        Self::with_urls(base_url, reverse_url)
    }

    pub fn with_urls(base_url: impl Into<String>, reverse_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let reverse_url = reverse_url.into();
        let client_builder = Client::builder()
            .user_agent(concat!("terminal-weather/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(8));
        let client =
            apply_loopback_proxy_policy(client_builder, &[base_url.as_str(), reverse_url.as_str()])
                .build()
                .context("failed to build geocode client")?;
        Ok(Self {
            client,
            base_url,
            reverse_url,
        })
    }

    pub async fn resolve(
        &self,
        city: String,
        country_code: Option<String>,
    ) -> Result<GeocodeResolution> {
        let country_code = country_code.as_deref();
        validate_resolve_inputs(&city, country_code)?;
        let payload = self.fetch_geocode_response(&city, country_code).await?;
        Ok(classify_resolution(city, country_code, payload))
    }

    pub async fn reverse_resolve(&self, latitude: f64, longitude: f64) -> Result<Option<Location>> {
        let mut response = self
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

        let mut body_bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .context("reading reverse geocoding chunk failed")?
        {
            if body_bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                anyhow::bail!("reverse geocoding response too large");
            }
            body_bytes.extend_from_slice(&chunk);
        }

        let payload: ReverseGeocodeResponse = serde_json::from_slice(&body_bytes)
            .context("failed to decode reverse geocoding response")?;

        Ok(payload
            .address
            .and_then(|address| location_from_reverse_address(address, latitude, longitude)))
    }

    async fn fetch_geocode_response(
        &self,
        city: &str,
        country_code: Option<&str>,
    ) -> Result<GeocodeResponse> {
        let request = self.build_geocode_request(city, country_code);
        let mut response = request
            .send()
            .await
            .context("geocoding request failed")?
            .error_for_status()
            .context("geocoding request returned non-success status")?;

        let mut body_bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .context("reading geocoding chunk failed")?
        {
            if body_bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                anyhow::bail!("geocoding response too large");
            }
            body_bytes.extend_from_slice(&chunk);
        }

        serde_json::from_slice(&body_bytes).context("failed to decode geocoding response")
    }

    fn build_geocode_request(
        &self,
        city: &str,
        country_code: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let mut request = self.client.get(&self.base_url).query(&[
            ("name", city),
            ("count", "5"),
            ("language", "en"),
            ("format", "json"),
        ]);

        if let Some(code) = country_code {
            request = request.query(&[("countryCode", code)]);
        }

        request
    }
}

fn no_geocode_results(results: Option<&Vec<GeocodeResult>>) -> bool {
    results.is_none_or(Vec::is_empty)
}

fn validate_resolve_inputs(city: &str, country_code: Option<&str>) -> Result<()> {
    if city.chars().count() > 100 {
        anyhow::bail!("City name is too long (max 100 chars)");
    }
    if country_code.is_some_and(|code| code.chars().count() > 10) {
        anyhow::bail!("Country code is too long (max 10 chars)");
    }
    Ok(())
}

fn classify_resolution(
    city: String,
    country_code: Option<&str>,
    payload: GeocodeResponse,
) -> GeocodeResolution {
    if no_geocode_results(payload.results.as_ref()) {
        return GeocodeResolution::NotFound(city);
    }

    let mut ranked = rank_locations(payload.results.unwrap_or_default(), &city, country_code);
    let top = ranked.remove(0);

    // OPTIMIZATION: Take ownership of `top` and `ranked` to avoid cloning `Location` strings.
    match ambiguous_options(top, ranked) {
        Ok(options) => GeocodeResolution::NeedsDisambiguation(options),
        Err(top_location) => GeocodeResolution::Selected(*top_location),
    }
}

fn ambiguous_options(
    top: ScoredLocation,
    ranked: Vec<ScoredLocation>,
) -> Result<Vec<Location>, Box<Location>> {
    let Some(second) = ranked.first() else {
        return Err(Box::new(top.location));
    };
    if !is_ambiguous(&top, second) {
        return Err(Box::new(top.location));
    }
    let mut options = vec![top.location];
    options.extend(ranked.into_iter().map(|s| s.location).take(4));
    Ok(options)
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
        name: sanitize_text(&entry.name),
        latitude: entry.latitude,
        longitude: entry.longitude,
        country: entry.country.map(|s| sanitize_text(&s)),
        admin1: entry.admin1.map(|s| sanitize_text(&s)),
        timezone: entry.timezone.map(|s| sanitize_text(&s)),
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
        name: sanitize_text(&name),
        latitude,
        longitude,
        country: address.country.map(|s| sanitize_text(&s)),
        admin1: address.state.map(|s| sanitize_text(&s)),
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
mod tests;

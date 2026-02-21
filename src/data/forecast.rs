#![allow(clippy::missing_errors_doc)]

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{
    AirQualityReading, CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location,
    parse_date, parse_datetime,
};

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";
const FORECAST_URL_ENV: &str = "TERMINAL_WEATHER_FORECAST_URL";
const AIR_QUALITY_URL_ENV: &str = "TERMINAL_WEATHER_AIR_QUALITY_URL";

#[derive(Debug, Clone)]
pub struct ForecastClient {
    client: Client,
    base_url: String,
    air_quality_url: String,
}

impl Default for ForecastClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ForecastClient {
    #[must_use]
    pub fn new() -> Self {
        let (base_url, air_quality_url) = resolve_api_urls(|key| std::env::var(key).ok());
        Self::with_urls(base_url, air_quality_url)
    }

    #[must_use]
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let (_, air_quality_url) = resolve_api_urls(|key| std::env::var(key).ok());
        Self::with_urls(base_url, air_quality_url)
    }

    #[must_use]
    pub fn with_urls(base_url: impl Into<String>, air_quality_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: base_url.into(),
            air_quality_url: air_quality_url.into(),
        }
    }

    #[must_use]
    pub fn with_air_quality_url(mut self, url: impl Into<String>) -> Self {
        self.air_quality_url = url.into();
        self
    }

    pub async fn fetch(&self, location: Location) -> Result<ForecastBundle> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&forecast_query(&location))
            .send()
            .await
            .context("forecast request failed")?
            .error_for_status()
            .context("forecast request returned non-success status")?;

        let payload: ForecastResponse = response
            .json()
            .await
            .context("failed to parse forecast payload")?;

        let daily = parse_daily(&payload.daily);
        let current = current_from_payload(&payload, &daily);
        let air_quality = self.fetch_air_quality(&location).await;

        Ok(ForecastBundle {
            location,
            current,
            hourly: parse_hourly(&payload.hourly),
            daily,
            air_quality,
            fetched_at: Utc::now(),
        })
    }

    async fn fetch_air_quality(&self, location: &Location) -> Option<AirQualityReading> {
        let response = self
            .client
            .get(&self.air_quality_url)
            .query(&air_quality_query(location))
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let payload: AirQualityResponse = response.json().await.ok()?;
        parse_air_quality(payload.current.as_ref())
    }
}

fn resolve_api_urls(get_env: impl Fn(&str) -> Option<String>) -> (String, String) {
    let forecast_url = get_env(FORECAST_URL_ENV).unwrap_or_else(|| FORECAST_URL.to_string());
    let air_quality_url =
        get_env(AIR_QUALITY_URL_ENV).unwrap_or_else(|| AIR_QUALITY_URL.to_string());
    (forecast_url, air_quality_url)
}

fn forecast_query(location: &Location) -> Vec<(&'static str, String)> {
    vec![
        ("latitude", location.latitude.to_string()),
        ("longitude", location.longitude.to_string()),
        ("current", CurrentBlock::field_names().join(",")),
        (
            "hourly",
            HourlyBlock::field_names()
                .iter()
                .filter(|&&f| f != "time")
                .copied()
                .collect::<Vec<_>>()
                .join(","),
        ),
        (
            "daily",
            DailyBlock::field_names()
                .iter()
                .filter(|&&f| f != "time")
                .copied()
                .collect::<Vec<_>>()
                .join(","),
        ),
        ("timezone", "auto".to_string()),
        ("forecast_days", "7".to_string()),
        ("forecast_hours", "48".to_string()),
    ]
}

fn air_quality_query(location: &Location) -> Vec<(&'static str, String)> {
    vec![
        ("latitude", location.latitude.to_string()),
        ("longitude", location.longitude.to_string()),
        ("current", "us_aqi,european_aqi".to_string()),
        ("timezone", "auto".to_string()),
    ]
}

fn current_from_payload(payload: &ForecastResponse, daily: &[DailyForecast]) -> CurrentConditions {
    CurrentConditions {
        temperature_2m_c: payload.current.temperature_2m,
        relative_humidity_2m: payload.current.relative_humidity_2m,
        apparent_temperature_c: payload.current.apparent_temperature,
        dew_point_2m_c: payload.current.dew_point_2m,
        weather_code: payload.current.weather_code,
        precipitation_mm: payload.current.precipitation,
        cloud_cover: payload.current.cloud_cover,
        pressure_msl_hpa: payload.current.pressure_msl,
        visibility_m: payload.current.visibility,
        wind_speed_10m: payload.current.wind_speed_10m,
        wind_gusts_10m: payload.current.wind_gusts_10m,
        wind_direction_10m: payload.current.wind_direction_10m,
        is_day: payload.current.is_day == 1,
        high_today_c: daily.first().and_then(|d| d.temperature_max_c),
        low_today_c: daily.first().and_then(|d| d.temperature_min_c),
    }
}

fn parse_hourly(hourly: &HourlyBlock) -> Vec<HourlyForecast> {
    parse_time_series(&hourly.time, parse_datetime, |idx, time| HourlyForecast {
        time,
        temperature_2m_c: hourly.temperature_2m.get(idx).copied().flatten(),
        weather_code: hourly.weather_code.get(idx).copied().flatten(),
        is_day: hourly
            .is_day
            .get(idx)
            .copied()
            .flatten()
            .map(|value| value == 1),
        relative_humidity_2m: hourly.relative_humidity_2m.get(idx).copied().flatten(),
        precipitation_probability: hourly.precipitation_probability.get(idx).copied().flatten(),
        precipitation_mm: hourly.precipitation.get(idx).copied().flatten(),
        rain_mm: hourly.rain.get(idx).copied().flatten(),
        snowfall_cm: hourly.snowfall.get(idx).copied().flatten(),
        wind_speed_10m: hourly.wind_speed_10m.get(idx).copied().flatten(),
        wind_gusts_10m: hourly.wind_gusts_10m.get(idx).copied().flatten(),
        pressure_msl_hpa: hourly.pressure_msl.get(idx).copied().flatten(),
        visibility_m: hourly.visibility.get(idx).copied().flatten(),
        cloud_cover: hourly.cloud_cover.get(idx).copied().flatten(),
        cloud_cover_low: hourly.cloud_cover_low.get(idx).copied().flatten(),
        cloud_cover_mid: hourly.cloud_cover_mid.get(idx).copied().flatten(),
        cloud_cover_high: hourly.cloud_cover_high.get(idx).copied().flatten(),
    })
}

fn parse_daily(daily: &DailyBlock) -> Vec<DailyForecast> {
    parse_time_series(&daily.time, parse_date, |idx, date| DailyForecast {
        date,
        weather_code: daily.weather_code.get(idx).copied().flatten(),
        temperature_max_c: daily.temperature_2m_max.get(idx).copied().flatten(),
        temperature_min_c: daily.temperature_2m_min.get(idx).copied().flatten(),
        sunrise: daily.sunrise.get(idx).and_then(|v| parse_datetime(v)),
        sunset: daily.sunset.get(idx).and_then(|v| parse_datetime(v)),
        uv_index_max: daily.uv_index_max.get(idx).copied().flatten(),
        precipitation_probability_max: daily
            .precipitation_probability_max
            .get(idx)
            .copied()
            .flatten(),
        precipitation_sum_mm: daily.precipitation_sum.get(idx).copied().flatten(),
        rain_sum_mm: daily.rain_sum.get(idx).copied().flatten(),
        snowfall_sum_cm: daily.snowfall_sum.get(idx).copied().flatten(),
        precipitation_hours: daily.precipitation_hours.get(idx).copied().flatten(),
        wind_gusts_10m_max: daily.wind_gusts_10m_max.get(idx).copied().flatten(),
        daylight_duration_s: daily.daylight_duration.get(idx).copied().flatten(),
        sunshine_duration_s: daily.sunshine_duration.get(idx).copied().flatten(),
    })
}

fn parse_air_quality(current: Option<&AirQualityCurrentBlock>) -> Option<AirQualityReading> {
    let current = current?;
    AirQualityReading::from_indices(current.us_aqi, current.european_aqi)
}

fn parse_time_series<T, R>(
    time_strings: &[String],
    parse_time: impl Fn(&str) -> Option<T>,
    map_fn: impl Fn(usize, T) -> R,
) -> Vec<R> {
    let mut out = Vec::with_capacity(time_strings.len());
    for (idx, time_str) in time_strings.iter().enumerate() {
        if let Some(time) = parse_time(time_str) {
            out.push(map_fn(idx, time));
        }
    }
    out
}

macro_rules! api_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        struct $name {
            $(
                $(#[$field_meta])*
                $field: $type
            ),*
        }

        impl $name {
            #[allow(dead_code)]
            fn field_names() -> &'static [&'static str] {
                &[
                    $(stringify!($field)),*
                ]
            }
        }
    };
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    current: CurrentBlock,
    hourly: HourlyBlock,
    daily: DailyBlock,
}

#[derive(Debug, Deserialize)]
struct AirQualityResponse {
    current: Option<AirQualityCurrentBlock>,
}

#[derive(Debug, Deserialize)]
struct AirQualityCurrentBlock {
    #[serde(default)]
    us_aqi: Option<f32>,
    #[serde(default)]
    european_aqi: Option<f32>,
}

api_struct! {
    #[derive(Debug, Deserialize)]
    struct CurrentBlock {
        temperature_2m: f32,
        relative_humidity_2m: f32,
        apparent_temperature: f32,
        dew_point_2m: f32,
        weather_code: u8,
        precipitation: f32,
        cloud_cover: f32,
        pressure_msl: f32,
        visibility: f32,
        wind_speed_10m: f32,
        wind_gusts_10m: f32,
        wind_direction_10m: f32,
        is_day: u8,
    }
}

api_struct! {
    #[derive(Debug, Deserialize)]
    struct HourlyBlock {
        time: Vec<String>,
        temperature_2m: Vec<Option<f32>>,
        weather_code: Vec<Option<u8>>,
        is_day: Vec<Option<u8>>,
        relative_humidity_2m: Vec<Option<f32>>,
        precipitation_probability: Vec<Option<f32>>,
        precipitation: Vec<Option<f32>>,
        rain: Vec<Option<f32>>,
        snowfall: Vec<Option<f32>>,
        wind_speed_10m: Vec<Option<f32>>,
        wind_gusts_10m: Vec<Option<f32>>,
        pressure_msl: Vec<Option<f32>>,
        visibility: Vec<Option<f32>>,
        cloud_cover: Vec<Option<f32>>,
        cloud_cover_low: Vec<Option<f32>>,
        cloud_cover_mid: Vec<Option<f32>>,
        cloud_cover_high: Vec<Option<f32>>,
    }
}

api_struct! {
    #[derive(Debug, Deserialize)]
    struct DailyBlock {
        time: Vec<String>,
        weather_code: Vec<Option<u8>>,
        temperature_2m_max: Vec<Option<f32>>,
        temperature_2m_min: Vec<Option<f32>>,
        sunrise: Vec<String>,
        sunset: Vec<String>,
        uv_index_max: Vec<Option<f32>>,
        precipitation_probability_max: Vec<Option<f32>>,
        precipitation_sum: Vec<Option<f32>>,
        rain_sum: Vec<Option<f32>>,
        snowfall_sum: Vec<Option<f32>>,
        precipitation_hours: Vec<Option<f32>>,
        wind_gusts_10m_max: Vec<Option<f32>>,
        daylight_duration: Vec<Option<f32>>,
        sunshine_duration: Vec<Option<f32>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hourly_skips_bad_timestamps() {
        let block = HourlyBlock {
            time: vec!["bad".to_string(), "2026-02-12T10:00".to_string()],
            temperature_2m: vec![Some(1.0), Some(2.0)],
            weather_code: vec![Some(0), Some(1)],
            is_day: vec![Some(1), Some(0)],
            relative_humidity_2m: vec![Some(50.0), Some(60.0)],
            precipitation_probability: vec![Some(10.0), Some(20.0)],
            precipitation: vec![Some(0.0), Some(0.2)],
            rain: vec![Some(0.0), Some(0.2)],
            snowfall: vec![Some(0.0), Some(0.0)],
            wind_speed_10m: vec![Some(5.0), Some(6.0)],
            wind_gusts_10m: vec![Some(8.0), Some(10.0)],
            pressure_msl: vec![Some(1002.0), Some(1003.0)],
            visibility: vec![Some(9000.0), Some(8500.0)],
            cloud_cover: vec![Some(35.0), Some(40.0)],
            cloud_cover_low: vec![Some(12.0), Some(15.0)],
            cloud_cover_mid: vec![Some(20.0), Some(22.0)],
            cloud_cover_high: vec![Some(30.0), Some(35.0)],
        };

        let parsed = parse_hourly(&block);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].is_day, Some(false));
    }

    #[test]
    fn parse_air_quality_prefers_us_index() {
        let current = AirQualityCurrentBlock {
            us_aqi: Some(57.0),
            european_aqi: Some(16.0),
        };

        let parsed = parse_air_quality(Some(&current)).expect("aqi reading");
        assert_eq!(parsed.us_aqi, Some(57));
        assert_eq!(parsed.european_aqi, Some(16));
    }

    #[test]
    fn parse_air_quality_returns_none_when_missing() {
        assert!(parse_air_quality(None).is_none());
    }

    #[test]
    fn resolve_api_urls_uses_defaults_when_env_missing() {
        let (forecast_url, air_quality_url) = resolve_api_urls(|_| None);
        assert_eq!(forecast_url, FORECAST_URL);
        assert_eq!(air_quality_url, AIR_QUALITY_URL);
    }

    #[test]
    fn resolve_api_urls_prefers_env_values() {
        let (forecast_url, air_quality_url) = resolve_api_urls(|key| match key {
            FORECAST_URL_ENV => Some("https://example.com/forecast".to_string()),
            AIR_QUALITY_URL_ENV => Some("https://example.com/aq".to_string()),
            _ => None,
        });
        assert_eq!(forecast_url, "https://example.com/forecast");
        assert_eq!(air_quality_url, "https://example.com/aq");
    }

    #[test]
    fn with_urls_sets_both_endpoints() {
        let client =
            ForecastClient::with_urls("https://example.com/forecast", "https://example.com/aq");
        assert_eq!(client.base_url, "https://example.com/forecast");
        assert_eq!(client.air_quality_url, "https://example.com/aq");
    }
}

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::weather::{
    CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location, parse_date,
    parse_datetime,
};

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Debug, Clone)]
pub struct ForecastClient {
    client: Client,
    base_url: String,
}

impl Default for ForecastClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ForecastClient {
    pub fn new() -> Self {
        Self::with_base_url(FORECAST_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }

    pub async fn fetch(&self, location: Location) -> Result<ForecastBundle> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("latitude", location.latitude.to_string()),
                ("longitude", location.longitude.to_string()),
                (
                    "current",
                    "temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m,is_day"
                        .to_string(),
                ),
                (
                    "hourly",
                    "temperature_2m,weather_code,relative_humidity_2m,precipitation_probability"
                        .to_string(),
                ),
                (
                    "daily",
                    "weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,uv_index_max,precipitation_probability_max"
                        .to_string(),
                ),
                ("timezone", "auto".to_string()),
                ("forecast_days", "7".to_string()),
                ("forecast_hours", "48".to_string()),
            ])
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
        let current = CurrentConditions {
            temperature_2m_c: payload.current.temperature_2m,
            relative_humidity_2m: payload.current.relative_humidity_2m,
            apparent_temperature_c: payload.current.apparent_temperature,
            weather_code: payload.current.weather_code,
            wind_speed_10m: payload.current.wind_speed_10m,
            wind_direction_10m: payload.current.wind_direction_10m,
            is_day: payload.current.is_day == 1,
            high_today_c: daily.first().and_then(|d| d.temperature_max_c),
            low_today_c: daily.first().and_then(|d| d.temperature_min_c),
        };

        Ok(ForecastBundle {
            location,
            current,
            hourly: parse_hourly(&payload.hourly),
            daily,
            fetched_at: Utc::now(),
        })
    }
}

fn parse_hourly(hourly: &HourlyBlock) -> Vec<HourlyForecast> {
    let mut out = Vec::new();
    for idx in 0..hourly.time.len() {
        let Some(time) = parse_datetime(&hourly.time[idx]) else {
            continue;
        };

        out.push(HourlyForecast {
            time,
            temperature_2m_c: hourly.temperature_2m.get(idx).copied().flatten(),
            weather_code: hourly.weather_code.get(idx).copied().flatten(),
            relative_humidity_2m: hourly.relative_humidity_2m.get(idx).copied().flatten(),
            precipitation_probability: hourly.precipitation_probability.get(idx).copied().flatten(),
        });
    }
    out
}

fn parse_daily(daily: &DailyBlock) -> Vec<DailyForecast> {
    let mut out = Vec::new();
    for idx in 0..daily.time.len() {
        let Some(date) = parse_date(&daily.time[idx]) else {
            continue;
        };

        out.push(DailyForecast {
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
        });
    }
    out
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    current: CurrentBlock,
    hourly: HourlyBlock,
    daily: DailyBlock,
}

#[derive(Debug, Deserialize)]
struct CurrentBlock {
    temperature_2m: f32,
    relative_humidity_2m: f32,
    apparent_temperature: f32,
    weather_code: u8,
    wind_speed_10m: f32,
    wind_direction_10m: f32,
    is_day: u8,
}

#[derive(Debug, Deserialize)]
struct HourlyBlock {
    time: Vec<String>,
    temperature_2m: Vec<Option<f32>>,
    weather_code: Vec<Option<u8>>,
    relative_humidity_2m: Vec<Option<f32>>,
    precipitation_probability: Vec<Option<f32>>,
}

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
            relative_humidity_2m: vec![Some(50.0), Some(60.0)],
            precipitation_probability: vec![Some(10.0), Some(20.0)],
        };

        let parsed = parse_hourly(&block);
        assert_eq!(parsed.len(), 1);
    }
}

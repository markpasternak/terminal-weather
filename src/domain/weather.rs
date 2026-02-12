use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{cli::IconMode, resilience::freshness::FreshnessState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub timezone: Option<String>,
    pub population: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SilhouetteArt {
    pub label: String,
    pub lines: Vec<String>,
}

impl Location {
    pub fn from_coords(lat: f64, lon: f64) -> Self {
        Self {
            name: format!("{lat:.4}, {lon:.4}"),
            latitude: lat,
            longitude: lon,
            country: None,
            admin1: None,
            timezone: None,
            population: None,
        }
    }

    pub fn display_name(&self) -> String {
        match (&self.admin1, &self.country) {
            (Some(admin), Some(country)) => format!("{}, {}, {}", self.name, admin, country),
            (None, Some(country)) => format!("{}, {}", self.name, country),
            _ => self.name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurrentConditions {
    pub temperature_2m_c: f32,
    pub relative_humidity_2m: f32,
    pub apparent_temperature_c: f32,
    pub weather_code: u8,
    pub wind_speed_10m: f32,
    pub wind_direction_10m: f32,
    pub is_day: bool,
    pub high_today_c: Option<f32>,
    pub low_today_c: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub time: NaiveDateTime,
    pub temperature_2m_c: Option<f32>,
    pub weather_code: Option<u8>,
    pub relative_humidity_2m: Option<f32>,
    pub precipitation_probability: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub date: NaiveDate,
    pub weather_code: Option<u8>,
    pub temperature_max_c: Option<f32>,
    pub temperature_min_c: Option<f32>,
    pub sunrise: Option<NaiveDateTime>,
    pub sunset: Option<NaiveDateTime>,
    pub uv_index_max: Option<f32>,
    pub precipitation_probability_max: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ForecastBundle {
    pub location: Location,
    pub current: CurrentConditions,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub fetched_at: DateTime<Utc>,
}

impl ForecastBundle {
    pub fn current_weather_code(&self) -> u8 {
        self.current.weather_code
    }

    pub fn current_temp(&self, units: Units) -> i32 {
        round_temp(convert_temp(self.current.temperature_2m_c, units))
    }

    pub fn high_low(&self, units: Units) -> Option<(i32, i32)> {
        Some((
            round_temp(convert_temp(self.current.high_today_c?, units)),
            round_temp(convert_temp(self.current.low_today_c?, units)),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherCategory {
    Clear,
    Cloudy,
    Rain,
    Snow,
    Fog,
    Thunder,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    None,
    Rain,
    Snow,
    Fog,
    Thunder,
}

pub fn weather_code_to_category(code: u8) -> WeatherCategory {
    match code {
        0 | 1 => WeatherCategory::Clear,
        2 | 3 => WeatherCategory::Cloudy,
        45 | 48 => WeatherCategory::Fog,
        51..=57 | 61..=67 | 80..=82 => WeatherCategory::Rain,
        71..=77 | 85..=86 => WeatherCategory::Snow,
        95 | 96 | 99 => WeatherCategory::Thunder,
        _ => WeatherCategory::Unknown,
    }
}

pub fn weather_code_to_particle(code: u8) -> ParticleKind {
    match weather_code_to_category(code) {
        WeatherCategory::Rain => ParticleKind::Rain,
        WeatherCategory::Snow => ParticleKind::Snow,
        WeatherCategory::Fog => ParticleKind::Fog,
        WeatherCategory::Thunder => ParticleKind::Thunder,
        WeatherCategory::Cloudy | WeatherCategory::Clear | WeatherCategory::Unknown => {
            ParticleKind::None
        }
    }
}

pub fn weather_label(code: u8) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 => "Fog",
        48 => "Depositing rime fog",
        51 => "Light drizzle",
        53 => "Moderate drizzle",
        55 => "Dense drizzle",
        56 => "Light freezing drizzle",
        57 => "Dense freezing drizzle",
        61 => "Slight rain",
        63 => "Moderate rain",
        65 => "Heavy rain",
        66 => "Light freezing rain",
        67 => "Heavy freezing rain",
        71 => "Slight snowfall",
        73 => "Moderate snowfall",
        75 => "Heavy snowfall",
        77 => "Snow grains",
        80 => "Slight rain showers",
        81 => "Moderate rain showers",
        82 => "Violent rain showers",
        85 => "Slight snow showers",
        86 => "Heavy snow showers",
        95 => "Thunderstorm",
        96 => "Thunderstorm + light hail",
        99 => "Thunderstorm + heavy hail",
        _ => "Unknown",
    }
}

pub fn weather_icon(code: u8, mode: IconMode) -> &'static str {
    match mode {
        IconMode::Ascii => match code {
            0 | 1 => "SUN",
            2 | 3 => "CLD",
            45 | 48 => "FOG",
            51..=57 | 61..=67 | 80..=82 => "RAN",
            71..=77 | 85..=86 => "SNW",
            95 | 96 | 99 => "THN",
            _ => "---",
        },
        IconMode::Emoji => match code {
            0 | 1 => "☀️",
            2 | 3 => "☁️",
            45 | 48 => "🌫️",
            51..=57 | 61..=67 | 80..=82 => "🌧️",
            71..=77 | 85..=86 => "🌨️",
            95 | 96 | 99 => "⛈️",
            _ => "☁️",
        },
        IconMode::Unicode => match code {
            0 | 1 => "☀",
            2 | 3 => "☁",
            45 | 48 => "░",
            51..=57 | 61..=67 | 80..=82 => "☂",
            71..=77 | 85..=86 => "❄",
            95 | 96 | 99 => "⚡",
            _ => "☁",
        },
    }
}

#[derive(Debug, Clone)]
pub enum GeocodeResolution {
    Selected(Location),
    NeedsDisambiguation(Vec<Location>),
    NotFound(String),
}

#[derive(Debug, Clone)]
pub struct RefreshMetadata {
    pub last_success: Option<DateTime<Utc>>,
    pub last_attempt: Option<DateTime<Utc>>,
    pub state: FreshnessState,
    pub consecutive_failures: u32,
}

impl Default for RefreshMetadata {
    fn default() -> Self {
        Self {
            last_success: None,
            last_attempt: None,
            state: FreshnessState::Stale,
            consecutive_failures: 0,
        }
    }
}

impl RefreshMetadata {
    pub fn mark_success(&mut self) {
        let now = Utc::now();
        self.last_attempt = Some(now);
        self.last_success = Some(now);
        self.consecutive_failures = 0;
        self.state = FreshnessState::Fresh;
    }

    pub fn mark_failure(&mut self) {
        self.last_attempt = Some(Utc::now());
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
    }

    pub fn age_minutes(&self) -> Option<i64> {
        self.last_success.map(|ts| (Utc::now() - ts).num_minutes())
    }
}

pub fn convert_temp(celsius: f32, units: Units) -> f32 {
    match units {
        Units::Celsius => celsius,
        Units::Fahrenheit => celsius * 1.8 + 32.0,
    }
}

pub fn round_temp(value: f32) -> i32 {
    value.round() as i32
}

pub fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").ok()
}

pub fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

pub fn evaluate_freshness(
    last_success: Option<DateTime<Utc>>,
    consecutive_failures: u32,
) -> FreshnessState {
    crate::resilience::freshness::evaluate_freshness(last_success, consecutive_failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freezing_drizzle_codes_have_labels() {
        assert_eq!(weather_label(56), "Light freezing drizzle");
        assert_eq!(weather_label(57), "Dense freezing drizzle");
    }

    #[test]
    fn fahrenheit_conversion_rounding() {
        assert_eq!(round_temp(convert_temp(0.0, Units::Fahrenheit)), 32);
        assert_eq!(round_temp(convert_temp(20.0, Units::Fahrenheit)), 68);
    }
}

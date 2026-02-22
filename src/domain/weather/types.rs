use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::resilience::freshness::FreshnessState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Celsius,
    Fahrenheit,
}

impl Units {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Celsius => "C",
            Self::Fahrenheit => "F",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Celsius => "Celsius",
            Self::Fahrenheit => "Fahrenheit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HourlyViewMode {
    #[default]
    Table,
    Hybrid,
    Chart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Daypart {
    Morning,
    Noon,
    Evening,
    Night,
}

impl Daypart {
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Morning, Self::Noon, Self::Evening, Self::Night]
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Noon => "Noon",
            Self::Evening => "Evening",
            Self::Night => "Night",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DaypartSummary {
    pub date: NaiveDate,
    pub daypart: Daypart,
    pub weather_code: u8,
    pub temp_min_c: Option<f32>,
    pub temp_max_c: Option<f32>,
    pub wind_min_kmh: Option<f32>,
    pub wind_max_kmh: Option<f32>,
    pub precip_sum_mm: f32,
    pub precip_probability_max: Option<f32>,
    pub visibility_median_m: Option<f32>,
    pub sample_count: usize,
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

impl Location {
    #[must_use]
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

    #[must_use]
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
    pub dew_point_2m_c: f32,
    pub weather_code: u8,
    pub precipitation_mm: f32,
    pub cloud_cover: f32,
    pub pressure_msl_hpa: f32,
    pub visibility_m: f32,
    pub wind_speed_10m: f32,
    pub wind_gusts_10m: f32,
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
    pub is_day: Option<bool>,
    pub relative_humidity_2m: Option<f32>,
    pub precipitation_probability: Option<f32>,
    pub precipitation_mm: Option<f32>,
    pub rain_mm: Option<f32>,
    pub snowfall_cm: Option<f32>,
    pub wind_speed_10m: Option<f32>,
    pub wind_gusts_10m: Option<f32>,
    pub pressure_msl_hpa: Option<f32>,
    pub visibility_m: Option<f32>,
    pub cloud_cover: Option<f32>,
    pub cloud_cover_low: Option<f32>,
    pub cloud_cover_mid: Option<f32>,
    pub cloud_cover_high: Option<f32>,
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
    pub precipitation_sum_mm: Option<f32>,
    pub rain_sum_mm: Option<f32>,
    pub snowfall_sum_cm: Option<f32>,
    pub precipitation_hours: Option<f32>,
    pub wind_gusts_10m_max: Option<f32>,
    pub daylight_duration_s: Option<f32>,
    pub sunshine_duration_s: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ForecastBundle {
    pub location: Location,
    pub current: CurrentConditions,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub air_quality: Option<AirQualityReading>,
    pub fetched_at: DateTime<Utc>,
}

impl ForecastBundle {
    #[must_use]
    pub fn current_weather_code(&self) -> u8 {
        self.current.weather_code
    }

    #[must_use]
    pub fn current_temp(&self, units: Units) -> i32 {
        super::round_temp(super::convert_temp(self.current.temperature_2m_c, units))
    }

    #[must_use]
    pub fn high_low(&self, units: Units) -> Option<(i32, i32)> {
        Some((
            super::round_temp(super::convert_temp(self.current.high_today_c?, units)),
            super::round_temp(super::convert_temp(self.current.low_today_c?, units)),
        ))
    }
}

pub const PRECIP_NEAR_TERM_HOURS: usize = 12;
pub const PRECIP_SIGNIFICANT_THRESHOLD_MM: f32 = 0.2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecipWindowSummary {
    pub first_idx: usize,
    pub first_amount_mm: f32,
    pub last_idx: usize,
    pub total_mm: f32,
}

impl PrecipWindowSummary {
    #[must_use]
    pub const fn has_precip_now(self) -> bool {
        self.first_idx == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirQualityCategory {
    Good,
    Moderate,
    UnhealthySensitive,
    Unhealthy,
    VeryUnhealthy,
    Hazardous,
    Unknown,
}

impl AirQualityCategory {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Moderate => "Moderate",
            Self::UnhealthySensitive => "USG",
            Self::Unhealthy => "Unhealthy",
            Self::VeryUnhealthy => "Very Unhealthy",
            Self::Hazardous => "Hazardous",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirQualityReading {
    pub us_aqi: Option<u16>,
    pub european_aqi: Option<u16>,
    pub category: AirQualityCategory,
}

impl AirQualityReading {
    #[must_use]
    pub fn from_indices(us_aqi: Option<f32>, european_aqi: Option<f32>) -> Option<Self> {
        let us_aqi = sanitize_aqi(us_aqi);
        let european_aqi = sanitize_aqi(european_aqi);
        if us_aqi.is_none() && european_aqi.is_none() {
            return None;
        }

        let category = us_aqi
            .map(categorize_us_aqi)
            .or_else(|| european_aqi.map(categorize_european_aqi))
            .unwrap_or(AirQualityCategory::Unknown);

        Some(Self {
            us_aqi,
            european_aqi,
            category,
        })
    }

    #[must_use]
    pub fn display_value(&self) -> String {
        self.us_aqi
            .or(self.european_aqi)
            .map_or_else(|| "N/A".to_string(), |value| value.to_string())
    }
}

fn sanitize_aqi(value: Option<f32>) -> Option<u16> {
    value
        .filter(|v| v.is_finite() && *v >= 0.0)
        .and_then(|v| u16::try_from(v.round() as i64).ok())
}

#[must_use]
pub fn categorize_us_aqi(aqi: u16) -> AirQualityCategory {
    match aqi {
        0..=50 => AirQualityCategory::Good,
        51..=100 => AirQualityCategory::Moderate,
        101..=150 => AirQualityCategory::UnhealthySensitive,
        151..=200 => AirQualityCategory::Unhealthy,
        201..=300 => AirQualityCategory::VeryUnhealthy,
        301..=500 => AirQualityCategory::Hazardous,
        _ => AirQualityCategory::Unknown,
    }
}

#[must_use]
pub fn categorize_european_aqi(aqi: u16) -> AirQualityCategory {
    match aqi {
        0..=20 => AirQualityCategory::Good,
        21..=40 => AirQualityCategory::Moderate,
        41..=60 => AirQualityCategory::UnhealthySensitive,
        61..=80 => AirQualityCategory::Unhealthy,
        81..=100 => AirQualityCategory::VeryUnhealthy,
        101.. => AirQualityCategory::Hazardous,
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
    pub next_retry_at: Option<DateTime<Utc>>,
    pub state: FreshnessState,
    pub consecutive_failures: u32,
}

impl Default for RefreshMetadata {
    fn default() -> Self {
        Self {
            last_success: None,
            last_attempt: None,
            next_retry_at: None,
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
        self.next_retry_at = None;
        self.consecutive_failures = 0;
        self.state = FreshnessState::Fresh;
    }

    pub fn mark_failure(&mut self) {
        self.last_attempt = Some(Utc::now());
        self.next_retry_at = None;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
    }

    pub fn schedule_retry_in(&mut self, delay_secs: u64) {
        let delay_secs = i64::try_from(delay_secs).unwrap_or(i64::MAX);
        self.next_retry_at = Some(Utc::now() + chrono::Duration::seconds(delay_secs));
    }

    pub fn clear_retry(&mut self) {
        self.next_retry_at = None;
    }

    #[must_use]
    pub fn age_minutes(&self) -> Option<i64> {
        self.last_success.map(|ts| (Utc::now() - ts).num_minutes())
    }

    #[must_use]
    pub fn retry_in_seconds(&self) -> Option<i64> {
        self.retry_in_seconds_at(Utc::now())
    }

    #[must_use]
    pub fn retry_in_seconds_at(&self, now: DateTime<Utc>) -> Option<i64> {
        self.next_retry_at
            .map(|retry_at| (retry_at - now).num_seconds().max(0))
    }
}

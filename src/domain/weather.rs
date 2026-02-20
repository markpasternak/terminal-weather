#![allow(clippy::cast_possible_truncation, clippy::must_use_candidate)]

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::{cli::IconMode, resilience::freshness::FreshnessState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Celsius,
    Fahrenheit,
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

#[must_use]
pub fn daypart_for_time(time: NaiveDateTime) -> Daypart {
    match time.hour() {
        6..=11 => Daypart::Morning,
        12..=17 => Daypart::Noon,
        18..=23 => Daypart::Evening,
        _ => Daypart::Night,
    }
}

#[must_use]
pub fn summarize_dayparts(
    hourly: &[HourlyForecast],
    fallback_weather_code: u8,
    max_days: usize,
) -> Vec<DaypartSummary> {
    if max_days == 0 || hourly.is_empty() {
        return Vec::new();
    }

    let dates = unique_dates(hourly, max_days);

    let mut out = Vec::with_capacity(dates.len() * Daypart::all().len());
    for date in dates {
        for part in Daypart::all() {
            out.push(summarize_date_daypart(
                hourly,
                date,
                part,
                fallback_weather_code,
            ));
        }
    }

    out
}

fn unique_dates(hourly: &[HourlyForecast], max_days: usize) -> Vec<NaiveDate> {
    let mut dates = Vec::<NaiveDate>::new();
    for hour in hourly {
        let date = hour.time.date();
        if !dates.contains(&date) {
            dates.push(date);
            if dates.len() >= max_days {
                break;
            }
        }
    }
    dates
}

fn summarize_date_daypart(
    hourly: &[HourlyForecast],
    date: NaiveDate,
    part: Daypart,
    fallback_weather_code: u8,
) -> DaypartSummary {
    let mut samples = Vec::new();
    let mut temp_values = Vec::new();
    let mut wind_values = Vec::new();
    let mut precip_values = Vec::new();
    let mut precip_prob_values = Vec::new();
    let mut visibility_values = Vec::new();

    for hour in hourly {
        if !matches_daypart(hour, date, part) {
            continue;
        }
        samples.push(hour);
        if let Some(value) = hour.temperature_2m_c {
            temp_values.push(value);
        }
        if let Some(value) = hour.wind_speed_10m {
            wind_values.push(value);
        }
        if let Some(value) = hour.precipitation_mm {
            precip_values.push(value.max(0.0));
        }
        if let Some(value) = hour.precipitation_probability {
            precip_prob_values.push(value);
        }
        if let Some(value) = hour.visibility_m {
            visibility_values.push(value);
        }
    }

    DaypartSummary {
        date,
        daypart: part,
        weather_code: dominant_weather_code(&samples, fallback_weather_code),
        temp_min_c: temp_values.iter().copied().min_by(f32::total_cmp),
        temp_max_c: temp_values.iter().copied().max_by(f32::total_cmp),
        wind_min_kmh: wind_values.iter().copied().min_by(f32::total_cmp),
        wind_max_kmh: wind_values.iter().copied().max_by(f32::total_cmp),
        precip_sum_mm: precip_values.into_iter().sum::<f32>(),
        precip_probability_max: precip_prob_values.into_iter().max_by(f32::total_cmp),
        visibility_median_m: median(visibility_values.into_iter()),
        sample_count: samples.len(),
    }
}

fn matches_daypart(hour: &HourlyForecast, date: NaiveDate, part: Daypart) -> bool {
    hour.time.date() == date && daypart_for_time(hour.time) == part
}

fn dominant_weather_code(samples: &[&HourlyForecast], fallback: u8) -> u8 {
    let mut counts = BTreeMap::<u8, usize>::new();
    for sample in samples {
        if let Some(code) = sample.weather_code {
            *counts.entry(code).or_default() += 1;
        }
    }

    counts
        .into_iter()
        .max_by(|(code_a, count_a), (code_b, count_b)| {
            count_a.cmp(count_b).then_with(|| code_b.cmp(code_a))
        })
        .map_or(fallback, |(code, _)| code)
}

fn median(values: impl Iterator<Item = f32>) -> Option<f32> {
    let mut items = values.collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }
    items.sort_by(f32::total_cmp);
    let mid = items.len() / 2;
    if items.len().is_multiple_of(2) {
        Some(f32::midpoint(items[mid - 1], items[mid]))
    } else {
        Some(items[mid])
    }
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
        round_temp(convert_temp(self.current.temperature_2m_c, units))
    }

    #[must_use]
    pub fn high_low(&self, units: Units) -> Option<(i32, i32)> {
        Some((
            round_temp(convert_temp(self.current.high_today_c?, units)),
            round_temp(convert_temp(self.current.low_today_c?, units)),
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

#[must_use]
pub fn summarize_precip_window(
    hourly: &[HourlyForecast],
    lookahead_hours: usize,
    threshold_mm: f32,
) -> Option<PrecipWindowSummary> {
    if lookahead_hours == 0 || !threshold_mm.is_finite() || threshold_mm < 0.0 {
        return None;
    }

    let mut first_idx = None;
    let mut first_amount_mm = 0.0_f32;
    let mut last_idx = None;
    let mut total_mm = 0.0_f32;

    for (idx, hour) in hourly.iter().take(lookahead_hours + 1).enumerate() {
        let amount_mm = hour.precipitation_mm.unwrap_or(0.0).max(0.0);
        if amount_mm < threshold_mm {
            continue;
        }
        if first_idx.is_none() {
            first_idx = Some(idx);
            first_amount_mm = amount_mm;
        }
        last_idx = Some(idx);
        total_mm += amount_mm;
    }

    Some(PrecipWindowSummary {
        first_idx: first_idx?,
        first_amount_mm,
        last_idx: last_idx?,
        total_mm,
    })
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

#[must_use]
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

#[must_use]
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

#[must_use]
pub fn weather_label(code: u8) -> &'static str {
    weather_label_for_time(code, true)
}

#[must_use]
pub fn weather_label_for_time(code: u8, is_day: bool) -> &'static str {
    match code {
        0 => {
            if is_day {
                "Clear sky"
            } else {
                "Clear night"
            }
        }
        1 => {
            if is_day {
                "Mainly clear"
            } else {
                "Mainly clear night"
            }
        }
        _ => weather_label_lookup(code).unwrap_or("Unknown"),
    }
}

#[must_use]
pub fn weather_icon(code: u8, mode: IconMode, is_day: bool) -> &'static str {
    let (ascii, emoji, unicode) = icon_tokens(weather_code_to_category(code), is_day);
    match mode {
        IconMode::Ascii => ascii,
        IconMode::Emoji => emoji,
        IconMode::Unicode => unicode,
    }
}

const WEATHER_LABELS: &[(u8, &str)] = &[
    (2, "Partly cloudy"),
    (3, "Overcast"),
    (45, "Fog"),
    (48, "Depositing rime fog"),
    (51, "Light drizzle"),
    (53, "Moderate drizzle"),
    (55, "Dense drizzle"),
    (56, "Light freezing drizzle"),
    (57, "Dense freezing drizzle"),
    (61, "Slight rain"),
    (63, "Moderate rain"),
    (65, "Heavy rain"),
    (66, "Light freezing rain"),
    (67, "Heavy freezing rain"),
    (71, "Slight snowfall"),
    (73, "Moderate snowfall"),
    (75, "Heavy snowfall"),
    (77, "Snow grains"),
    (80, "Slight rain showers"),
    (81, "Moderate rain showers"),
    (82, "Violent rain showers"),
    (85, "Slight snow showers"),
    (86, "Heavy snow showers"),
    (95, "Thunderstorm"),
    (96, "Thunderstorm + light hail"),
    (99, "Thunderstorm + heavy hail"),
];

fn weather_label_lookup(code: u8) -> Option<&'static str> {
    WEATHER_LABELS
        .iter()
        .find_map(|(candidate, label)| (*candidate == code).then_some(*label))
}

fn icon_tokens(
    category: WeatherCategory,
    is_day: bool,
) -> (&'static str, &'static str, &'static str) {
    if matches!(category, WeatherCategory::Clear) {
        return clear_icon_tokens(is_day);
    }
    non_clear_icon_tokens(category)
}

fn clear_icon_tokens(is_day: bool) -> (&'static str, &'static str, &'static str) {
    if is_day {
        ("SUN", "☀️", "☀")
    } else {
        ("MON", "🌙", "☾")
    }
}

fn non_clear_icon_tokens(category: WeatherCategory) -> (&'static str, &'static str, &'static str) {
    match category {
        WeatherCategory::Cloudy => ("CLD", "☁️", "☁"),
        WeatherCategory::Rain => ("RAN", "🌧️", "☂"),
        WeatherCategory::Snow => ("SNW", "🌨️", "❄"),
        WeatherCategory::Fog => ("FOG", "🌫️", "░"),
        WeatherCategory::Thunder => ("THN", "⛈️", "⚡"),
        WeatherCategory::Unknown | WeatherCategory::Clear => ("---", "☁️", "☁"),
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
        self.next_retry_at = Some(Utc::now() + Duration::seconds(delay_secs));
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

#[must_use]
pub fn convert_temp(celsius: f32, units: Units) -> f32 {
    match units {
        Units::Celsius => celsius,
        Units::Fahrenheit => celsius * 1.8 + 32.0,
    }
}

#[must_use]
pub fn convert_wind_speed(kmh: f32) -> f32 {
    kmh / 3.6
}

#[must_use]
pub fn round_wind_speed(kmh: f32) -> i32 {
    convert_wind_speed(kmh).round() as i32
}

#[must_use]
pub fn round_temp(value: f32) -> i32 {
    value.round() as i32
}

#[must_use]
pub fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").ok()
}

#[must_use]
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
mod tests;

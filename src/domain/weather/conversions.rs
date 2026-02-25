use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use super::Units;
use crate::resilience::freshness::FreshnessState;

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

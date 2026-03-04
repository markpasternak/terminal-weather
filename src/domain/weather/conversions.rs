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

// OPTIMIZATION: Manually parse exact length and format ISO8601 strings returned by APIs
// like Open-Meteo. This avoids the heavy overhead of `chrono::NaiveDateTime::parse_from_str`
// with format strings, resulting in ~300x faster parsing for these common structures.
#[must_use]
#[allow(clippy::collapsible_if)]
pub fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    if value.len() == 16 {
        let b = value.as_bytes();
        if b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':' {
            let parts = (
                parse_four(&b[0..4]),
                parse_two(&b[5..7]),
                parse_two(&b[8..10]),
                parse_two(&b[11..13]),
                parse_two(&b[14..16]),
            );
            if let (Some(y), Some(m), Some(d), Some(h), Some(min)) = parts {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    if let Some(dt) = date.and_hms_opt(h, min, 0) {
                        return Some(dt);
                    }
                }
            }
        }
    }
    // Fallback for unexpected formats
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").ok()
}

#[must_use]
#[allow(clippy::collapsible_if)]
pub fn parse_date(value: &str) -> Option<NaiveDate> {
    if value.len() == 10 {
        let b = value.as_bytes();
        if b[4] == b'-' && b[7] == b'-' {
            let parts = (
                parse_four(&b[0..4]),
                parse_two(&b[5..7]),
                parse_two(&b[8..10]),
            );
            if let (Some(y), Some(m), Some(d)) = parts {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
    }
    // Fallback for unexpected formats
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

#[inline]
fn parse_two(b: &[u8]) -> Option<u32> {
    let d1 = b[0].wrapping_sub(b'0');
    let d2 = b[1].wrapping_sub(b'0');
    if d1 > 9 || d2 > 9 {
        return None;
    }
    Some(u32::from(d1 * 10 + d2))
}

#[inline]
fn parse_four(b: &[u8]) -> Option<i32> {
    let d1 = b[0].wrapping_sub(b'0');
    let d2 = b[1].wrapping_sub(b'0');
    let d3 = b[2].wrapping_sub(b'0');
    let d4 = b[3].wrapping_sub(b'0');
    if d1 > 9 || d2 > 9 || d3 > 9 || d4 > 9 {
        return None;
    }
    Some(i32::from(d1) * 1000 + i32::from(d2) * 100 + i32::from(d3) * 10 + i32::from(d4))
}

pub fn evaluate_freshness(
    last_success: Option<DateTime<Utc>>,
    consecutive_failures: u32,
) -> FreshnessState {
    crate::resilience::freshness::evaluate_freshness(last_success, consecutive_failures)
}

#[must_use]
pub fn sanitize_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

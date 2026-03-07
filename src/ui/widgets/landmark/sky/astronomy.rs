use chrono::{Datelike, Timelike};

use crate::domain::weather::ForecastBundle;

const MOON_PHASE_THRESHOLDS: &[(f32, char)] = &[
    (0.06, '●'),
    (0.19, '◔'),
    (0.31, '◑'),
    (0.44, '◕'),
    (0.56, '○'),
    (0.69, '◖'),
    (0.81, '◐'),
    (0.94, '◗'),
    (1.0, '●'),
];

pub(super) fn sky_context_line(sunrise_h: f32, sunset_h: f32, now_h: f32, is_day: bool) -> String {
    if is_day {
        let remaining_mins = ((sunset_h - now_h) * 60.0).max(0.0).round() as i32;
        let hours = remaining_mins / 60;
        let mins = remaining_mins % 60;
        if remaining_mins <= 0 {
            "Sunset passing · twilight".to_string()
        } else {
            format!("{hours}h {mins:02}m of daylight remaining")
        }
    } else {
        let remaining_mins = if now_h > sunset_h {
            ((24.0 - now_h + sunrise_h) * 60.0).round() as i32
        } else {
            ((sunrise_h - now_h) * 60.0).round() as i32
        };
        let hours = remaining_mins.max(0) / 60;
        let mins = remaining_mins.max(0) % 60;
        format!("{hours}h {mins:02}m until sunrise")
    }
}

pub(super) fn sun_window(bundle: &ForecastBundle) -> (f32, f32) {
    bundle.daily.first().map_or((6.0, 18.0), |day| {
        (
            day.sunrise.map_or(6.0, |t| hm_to_hour_f32(&t)),
            day.sunset.map_or(18.0, |t| hm_to_hour_f32(&t)),
        )
    })
}

pub(super) fn current_hour(bundle: &ForecastBundle) -> f32 {
    bundle
        .hourly
        .first()
        .map_or(12.0, |hour| hm_to_hour_f32(&hour.time))
}

pub(super) fn celestial_progress(sunrise_h: f32, sunset_h: f32, now_h: f32, is_day: bool) -> f32 {
    if is_day {
        let daylight_span = (sunset_h - sunrise_h).rem_euclid(24.0).max(0.1);
        ((now_h - sunrise_h).rem_euclid(24.0) / daylight_span).clamp(0.0, 1.0)
    } else {
        let night_span = (sunrise_h - sunset_h).rem_euclid(24.0).max(0.1);
        let elapsed = (now_h - sunset_h).rem_euclid(24.0);
        (1.0 - (elapsed / night_span).clamp(0.0, 1.0)).clamp(0.0, 1.0)
    }
}

pub(super) fn moon_phase(bundle: &ForecastBundle) -> char {
    let day = bundle.daily.first().map_or(1, |d| d.date.ordinal()) as f32;
    let phase = (day % 29.53) / 29.53;
    for (threshold, symbol) in MOON_PHASE_THRESHOLDS {
        if phase < *threshold {
            return *symbol;
        }
    }
    '●'
}

pub(super) fn format_time_hm(hour_f: f32) -> String {
    let total = (hour_f * 60.0).round().max(0.0) as i32;
    let h = (total / 60).rem_euclid(24);
    let m = total % 60;
    format!("{h:02}:{m:02}")
}

pub(super) fn format_duration_hm(seconds: f32) -> String {
    let total_minutes = (seconds.max(0.0) / 60.0).round() as i64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours:02}:{minutes:02}")
}

pub(super) fn format_optional_duration_hm(seconds: Option<f32>) -> Option<String> {
    seconds.map(format_duration_hm)
}

pub(super) fn hm_to_hour_f32<T: Timelike>(value: &T) -> f32 {
    value.hour() as f32 + value.minute() as f32 / 60.0
}

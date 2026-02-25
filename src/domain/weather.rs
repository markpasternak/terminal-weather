#![allow(clippy::cast_possible_truncation, clippy::must_use_candidate)]

use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime, Timelike};

mod conditions;
mod conversions;
mod insights;
mod types;

pub use conditions::{
    ParticleKind, WeatherCategory, weather_code_to_category, weather_code_to_particle,
    weather_icon, weather_label, weather_label_for_time,
};
pub use conversions::{
    convert_temp, convert_wind_speed, evaluate_freshness, parse_date, parse_datetime, round_temp,
    round_wind_speed, sanitize_text,
};
pub use insights::{
    ActionCue, ChangeEvent, ChangeKind, InsightConfidence, NowcastInsight, ReliabilitySummary,
    derive_nowcast_insight, next_notable_change,
};
pub use types::{
    AirQualityCategory, AirQualityReading, CurrentConditions, DailyForecast, Daypart,
    DaypartSummary, ForecastBundle, GeocodeResolution, HourlyForecast, HourlyViewMode, Location,
    PRECIP_NEAR_TERM_HOURS, PRECIP_SIGNIFICANT_THRESHOLD_MM, PrecipWindowSummary, RefreshMetadata,
    Units, categorize_european_aqi, categorize_us_aqi,
};

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

#[must_use]
pub fn summarize_precip_window(
    hourly: &[HourlyForecast],
    lookahead_hours: usize,
    threshold_mm: f32,
) -> Option<PrecipWindowSummary> {
    if lookahead_hours == 0 || !threshold_mm.is_finite() || threshold_mm < 0.0 {
        return None;
    }

    let matching = hourly
        .iter()
        .take(lookahead_hours + 1)
        .enumerate()
        .filter_map(|(idx, hour)| {
            let amount_mm = hour.precipitation_mm.unwrap_or(0.0).max(0.0);
            (amount_mm >= threshold_mm).then_some((idx, amount_mm))
        })
        .collect::<Vec<_>>();

    let (first_idx, first_amount_mm) = *matching.first()?;
    let last_idx = matching.last()?.0;
    let total_mm = matching.iter().map(|(_, amount_mm)| amount_mm).sum::<f32>();

    Some(PrecipWindowSummary {
        first_idx,
        first_amount_mm,
        last_idx,
        total_mm,
    })
}

#[cfg(test)]
mod tests;

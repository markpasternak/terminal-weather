use chrono::{DateTime, Local, Utc};
use ratatui::style::Color;
use std::fmt::Write as _;

use crate::{
    app::state::AppState,
    domain::weather::{AirQualityCategory, ForecastBundle, HourlyForecast},
    resilience::freshness::FreshnessState,
    ui::theme::Theme,
};

pub(super) fn last_updated_label(
    state: &AppState,
    weather: &ForecastBundle,
    use_colon: bool,
) -> String {
    let timezone = weather.location.timezone.as_deref().unwrap_or("--");
    let with_success = |ts: DateTime<Utc>| {
        let local = ts.with_timezone(&Local);
        let mins = state.refresh_meta.age_minutes().unwrap_or(0);
        if use_colon {
            format!(
                "Last updated: {} local ({}m ago) · City TZ {}",
                local.format("%H:%M"),
                mins.max(0),
                timezone
            )
        } else {
            format!(
                "Last updated {} local ({}m ago) · City TZ {}",
                local.format("%H:%M"),
                mins.max(0),
                timezone
            )
        }
    };

    state.refresh_meta.last_success.map_or_else(
        || {
            if use_colon {
                format!("Last updated: --:-- local · City TZ {timezone}")
            } else {
                format!("Last updated --:-- local · City TZ {timezone}")
            }
        },
        with_success,
    )
}

pub(super) fn fetch_context_line(state: &AppState, max_len: usize) -> Option<String> {
    let error = state.last_error.as_ref()?;
    if matches!(state.refresh_meta.state, FreshnessState::Fresh) {
        return None;
    }
    let mut context = format!("Last fetch failed: {}", summarize_error(error, max_len));
    if let Some(retry_secs) = state.refresh_meta.retry_in_seconds() {
        let _ = write!(context, " · retry in {retry_secs}s");
    }
    Some(context)
}

pub(super) fn summarize_error(error: &str, max_len: usize) -> String {
    let first_line = error.lines().next().unwrap_or_default();
    let text = first_line.trim();
    if text.chars().count() <= max_len {
        return text.to_string();
    }

    // Performance Optimization: Truncate string by finding the correct byte boundary
    // corresponding to the target character count using char_indices. This allows a
    // single memory allocation and string slice, eliminating the overhead of iterating
    // over chars and repeatedly pushing to a new String.
    if let Some((byte_idx, _)) = text.char_indices().nth(max_len.saturating_sub(1)) {
        let mut out = String::with_capacity(byte_idx + 3);
        out.push_str(&text[..byte_idx]);
        out.push('…');
        return out;
    }
    text.to_string()
}

pub(super) fn next_precip_probability(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(12)
        .find_map(|hour| hour.precipitation_probability)
        .map_or_else(
            || "--".to_string(),
            |value| format!("{}%", value.round() as i32),
        )
}

pub(super) fn aqi_summary(weather: &ForecastBundle) -> (String, AirQualityCategory, bool) {
    let Some(reading) = weather.air_quality.as_ref() else {
        return ("N/A".to_string(), AirQualityCategory::Unknown, false);
    };

    (
        format!("{} {}", reading.display_value(), reading.category.label()),
        reading.category,
        true,
    )
}

pub(super) fn aqi_color(theme: Theme, category: AirQualityCategory, available: bool) -> Color {
    if !available {
        return theme.muted_text;
    }

    match category {
        AirQualityCategory::Good => theme.success,
        AirQualityCategory::Moderate => theme.warning,
        AirQualityCategory::UnhealthySensitive
        | AirQualityCategory::Unhealthy
        | AirQualityCategory::VeryUnhealthy
        | AirQualityCategory::Hazardous => theme.danger,
        AirQualityCategory::Unknown => theme.muted_text,
    }
}

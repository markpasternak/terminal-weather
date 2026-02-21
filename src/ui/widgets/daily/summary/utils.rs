pub(super) fn format_duration_hm(seconds: f32) -> String {
    let total_minutes = (seconds.max(0.0) / 60.0).round() as i64;
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    format!("{h:02}:{m:02}")
}

pub(super) fn profile_bar(values: &[f32], width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    crate::ui::widgets::shared::sparkline_blocks(values, width)
}

pub(super) fn precipitation_cue(day: &crate::domain::weather::DailyForecast) -> String {
    let precip = day.precipitation_sum_mm.unwrap_or(0.0);
    let snow = day.snowfall_sum_cm.unwrap_or(0.0);
    snow_cue(snow)
        .or_else(|| rain_cue(precip))
        .unwrap_or_else(|| "mostly dry".to_string())
}

pub(super) fn gust_cue(gust: f32) -> Option<String> {
    if gust >= 45.0 {
        return Some(format!(
            "gusty {}m/s",
            crate::domain::weather::round_wind_speed(gust)
        ));
    }
    if gust >= 30.0 {
        return Some(format!(
            "breezy {}m/s",
            crate::domain::weather::round_wind_speed(gust)
        ));
    }
    None
}

pub(super) fn sunlight_cue(day: &crate::domain::weather::DailyForecast) -> Option<&'static str> {
    let ratio = match (day.sunshine_duration_s, day.daylight_duration_s) {
        (Some(sun), Some(daylight)) if daylight > 0.0 => Some((sun / daylight).clamp(0.0, 1.0)),
        _ => None,
    }?;

    if ratio >= 0.65 {
        Some("bright")
    } else if ratio <= 0.25 {
        Some("dim")
    } else {
        None
    }
}

pub(super) fn first_day_time(
    bundle: &crate::domain::weather::ForecastBundle,
    projection: impl Fn(&crate::domain::weather::DailyForecast) -> Option<chrono::NaiveDateTime>,
) -> String {
    bundle.daily.first().and_then(projection).map_or_else(
        || "--:--".to_string(),
        |value| value.format("%H:%M").to_string(),
    )
}

pub(super) fn first_day_shifted_time(
    bundle: &crate::domain::weather::ForecastBundle,
    projection: impl Fn(&crate::domain::weather::DailyForecast) -> Option<chrono::NaiveDateTime>,
    shift_minutes: i64,
) -> String {
    bundle
        .daily
        .first()
        .and_then(projection)
        .map(|value| value + chrono::Duration::minutes(shift_minutes))
        .map_or_else(
            || "--:--".to_string(),
            |value| value.format("%H:%M").to_string(),
        )
}

pub(super) fn day_cue(day: &crate::domain::weather::DailyForecast) -> String {
    let mut parts = vec![precipitation_cue(day)];
    if let Some(gust) = gust_cue(day.wind_gusts_10m_max.unwrap_or(0.0)) {
        parts.push(gust);
    }
    if let Some(sunlight) = sunlight_cue(day) {
        parts.push(sunlight.to_string());
    }
    parts.join(", ")
}

fn snow_cue(snow: f32) -> Option<String> {
    (snow >= 1.0).then(|| format!("snow {snow:.1}cm"))
}

fn rain_cue(precip: f32) -> Option<String> {
    if precip >= 6.0 {
        Some(format!("wet {precip:.1}mm"))
    } else if precip >= 1.0 {
        Some(format!("light rain {precip:.1}mm"))
    } else {
        None
    }
}

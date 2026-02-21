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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::weather::DailyForecast;
    use chrono::NaiveDate;

    fn sample_day(
        precip_mm: f32,
        snow_cm: f32,
        gust_kmh: f32,
        sunshine_s: f32,
        daylight_s: f32,
    ) -> DailyForecast {
        DailyForecast {
            date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("date"),
            weather_code: Some(3),
            temperature_max_c: Some(8.0),
            temperature_min_c: Some(1.0),
            sunrise: None,
            sunset: None,
            uv_index_max: Some(3.0),
            precipitation_probability_max: Some(50.0),
            precipitation_sum_mm: Some(precip_mm),
            rain_sum_mm: Some(precip_mm),
            snowfall_sum_cm: Some(snow_cm),
            precipitation_hours: Some(1.0),
            wind_gusts_10m_max: Some(gust_kmh),
            daylight_duration_s: Some(daylight_s),
            sunshine_duration_s: Some(sunshine_s),
        }
    }

    // ── format_duration_hm ───────────────────────────────────────────────────

    #[test]
    fn format_duration_hm_zero_is_double_zero() {
        assert_eq!(format_duration_hm(0.0), "00:00");
    }

    #[test]
    fn format_duration_hm_rounds_to_minutes() {
        // 3661 seconds = 61 min = 1h01m
        assert_eq!(format_duration_hm(3661.0), "01:01");
    }

    #[test]
    fn format_duration_hm_negative_clamps_to_zero() {
        assert_eq!(format_duration_hm(-100.0), "00:00");
    }

    // ── profile_bar ──────────────────────────────────────────────────────────

    #[test]
    fn profile_bar_empty_returns_empty() {
        assert_eq!(profile_bar(&[], 5), String::new());
    }

    #[test]
    fn profile_bar_returns_correct_width() {
        let out = profile_bar(&[1.0, 2.0, 3.0], 6);
        assert_eq!(out.chars().count(), 6);
    }

    // ── precipitation_cue ────────────────────────────────────────────────────

    #[test]
    fn precipitation_cue_snow_takes_priority() {
        let day = sample_day(10.0, 5.0, 0.0, 0.0, 1.0);
        let cue = precipitation_cue(&day);
        assert!(cue.contains("snow"), "got: {cue}");
    }

    #[test]
    fn precipitation_cue_heavy_rain() {
        let day = sample_day(8.0, 0.0, 0.0, 0.0, 1.0);
        let cue = precipitation_cue(&day);
        assert!(cue.contains("wet"), "got: {cue}");
    }

    #[test]
    fn precipitation_cue_light_rain() {
        let day = sample_day(2.0, 0.0, 0.0, 0.0, 1.0);
        let cue = precipitation_cue(&day);
        assert!(cue.contains("light rain"), "got: {cue}");
    }

    #[test]
    fn precipitation_cue_dry() {
        let day = sample_day(0.0, 0.0, 0.0, 0.0, 1.0);
        let cue = precipitation_cue(&day);
        assert_eq!(cue, "mostly dry");
    }

    // ── gust_cue ─────────────────────────────────────────────────────────────

    #[test]
    fn gust_cue_none_below_threshold() {
        assert!(gust_cue(20.0).is_none());
    }

    #[test]
    fn gust_cue_breezy_threshold() {
        let cue = gust_cue(35.0).expect("should be Some");
        assert!(cue.contains("breezy"), "got: {cue}");
    }

    #[test]
    fn gust_cue_gusty_threshold() {
        let cue = gust_cue(50.0).expect("should be Some");
        assert!(cue.contains("gusty"), "got: {cue}");
    }

    // ── sunlight_cue ─────────────────────────────────────────────────────────

    #[test]
    fn sunlight_cue_none_when_data_missing() {
        let mut day = sample_day(0.0, 0.0, 0.0, 0.0, 0.0);
        day.sunshine_duration_s = None;
        assert!(sunlight_cue(&day).is_none());
    }

    #[test]
    fn sunlight_cue_bright() {
        let day = sample_day(0.0, 0.0, 0.0, 72_000.0, 100_000.0);
        // ratio = 0.72 >= 0.65 → bright
        assert_eq!(sunlight_cue(&day), Some("bright"));
    }

    #[test]
    fn sunlight_cue_dim() {
        let day = sample_day(0.0, 0.0, 0.0, 5_000.0, 40_000.0);
        // ratio = 0.125 <= 0.25 → dim
        assert_eq!(sunlight_cue(&day), Some("dim"));
    }

    #[test]
    fn sunlight_cue_none_for_mid_range() {
        let day = sample_day(0.0, 0.0, 0.0, 30_000.0, 70_000.0);
        // ratio ≈ 0.43, between 0.25 and 0.65 → None
        assert!(sunlight_cue(&day).is_none());
    }

    #[test]
    fn sunlight_cue_none_when_daylight_is_zero() {
        // Guard `daylight > 0.0` fails → treated as missing data
        let day = sample_day(0.0, 0.0, 0.0, 5_000.0, 0.0);
        assert!(sunlight_cue(&day).is_none());
    }

    // ── first_day_time / first_day_shifted_time ──────────────────────────────

    #[test]
    fn first_day_time_returns_placeholder_when_no_daily() {
        let mut bundle = crate::test_support::sample_bundle();
        bundle.daily.clear();
        let result = first_day_time(&bundle, |_| None);
        assert_eq!(result, "--:--");
    }

    #[test]
    fn first_day_time_returns_formatted_time() {
        use chrono::NaiveDateTime;
        let mut bundle = crate::test_support::sample_bundle();
        let dt = NaiveDateTime::parse_from_str("2026-02-12T07:15", "%Y-%m-%dT%H:%M").expect("dt");
        bundle.daily[0].sunrise = Some(dt);
        let result = first_day_time(&bundle, |d| d.sunrise);
        assert_eq!(result, "07:15");
    }

    #[test]
    fn first_day_shifted_time_applies_offset() {
        use chrono::NaiveDateTime;
        let mut bundle = crate::test_support::sample_bundle();
        let dt = NaiveDateTime::parse_from_str("2026-02-12T07:00", "%Y-%m-%dT%H:%M").expect("dt");
        bundle.daily[0].sunrise = Some(dt);
        // +30 minutes
        let result = first_day_shifted_time(&bundle, |d| d.sunrise, 30);
        assert_eq!(result, "07:30");
    }

    #[test]
    fn first_day_shifted_time_placeholder_when_no_value() {
        let bundle = crate::test_support::sample_bundle();
        let result = first_day_shifted_time(&bundle, |_| None, 15);
        assert_eq!(result, "--:--");
    }

    // ── day_cue ──────────────────────────────────────────────────────────────

    #[test]
    fn day_cue_combines_precip_gust_and_sun() {
        let day = sample_day(8.0, 0.0, 50.0, 72_000.0, 100_000.0);
        let cue = day_cue(&day);
        assert!(cue.contains("wet"), "got: {cue}");
        assert!(cue.contains("gusty"), "got: {cue}");
        assert!(cue.contains("bright"), "got: {cue}");
    }

    #[test]
    fn day_cue_dry_calm_returns_mostly_dry() {
        let day = sample_day(0.0, 0.0, 10.0, 30_000.0, 70_000.0);
        let cue = day_cue(&day);
        assert_eq!(cue, "mostly dry");
    }
}

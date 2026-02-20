use crate::cli::IconMode;
use chrono::Timelike;

use crate::domain::weather::{
    ForecastBundle, PRECIP_NEAR_TERM_HOURS, PRECIP_SIGNIFICANT_THRESHOLD_MM, PrecipWindowSummary,
    Units, WeatherCategory, convert_temp, round_temp, summarize_precip_window, weather_icon,
};
use crate::ui::widgets::landmark::shared::paint_char;

pub(super) fn atmos_context_line(
    bundle: &ForecastBundle,
    units: Units,
    category: WeatherCategory,
) -> String {
    if let Some(summary) = precip_summary(bundle) {
        return precip_context(bundle, summary);
    }
    stable_weather_context(bundle, units, category)
}

pub(super) fn paint_hud_badge(
    canvas: &mut [Vec<char>],
    bundle: &ForecastBundle,
    units: Units,
    width: usize,
) {
    if width < 30 || canvas.len() < 4 {
        return;
    }
    let badge = hud_badge_text(bundle, units);
    let start_x = width.saturating_sub(badge.chars().count() + 1);
    for (idx, ch) in badge.chars().enumerate() {
        paint_char(canvas, (start_x + idx) as isize, 0, ch, true);
    }
}

fn precip_summary(bundle: &ForecastBundle) -> Option<PrecipWindowSummary> {
    summarize_precip_window(
        &bundle.hourly,
        PRECIP_NEAR_TERM_HOURS,
        PRECIP_SIGNIFICANT_THRESHOLD_MM,
    )
}

fn precip_context(bundle: &ForecastBundle, summary: PrecipWindowSummary) -> String {
    let now_hour = bundle
        .hourly
        .first()
        .map_or(12, |hour| hour.time.hour() as usize);
    let end_hour = (now_hour + summary.last_idx + 1) % 24;
    if summary.has_precip_now() {
        format!(
            "Precip clearing by {end_hour:02}:00 · {:.0}mm expected",
            summary.total_mm
        )
    } else {
        format!(
            "Precipitation expected through {end_hour:02}:00 · {:.0}mm",
            summary.total_mm
        )
    }
}

fn stable_weather_context(
    bundle: &ForecastBundle,
    units: Units,
    category: WeatherCategory,
) -> String {
    if matches!(category, WeatherCategory::Snow) {
        return "Snow conditions · dress warm".to_string();
    }
    if matches!(category, WeatherCategory::Fog) {
        return "Low visibility · fog advisory".to_string();
    }
    if matches!(category, WeatherCategory::Thunder) {
        return "Thunderstorm conditions · stay alert".to_string();
    }
    if matches!(category, WeatherCategory::Clear) {
        return clear_context(bundle);
    }
    let (temp, unit) = temp_display(bundle.current.temperature_2m_c, units);
    if matches!(category, WeatherCategory::Unknown) {
        return format!("Currently {temp}°{unit}");
    }
    format!("Currently {temp}°{unit} · overcast skies")
}

fn clear_context(bundle: &ForecastBundle) -> String {
    if !bundle.current.is_day {
        return "Clear night · great for stargazing".to_string();
    }
    let uv = bundle
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .unwrap_or(0.0);
    if uv > 5.0 {
        format!("Clear skies · UV {uv:.1} high — sunscreen advised")
    } else {
        "Clear skies · enjoy the day".to_string()
    }
}

fn hud_badge_text(bundle: &ForecastBundle, units: Units) -> String {
    let (temp, unit) = temp_display(bundle.current.temperature_2m_c, units);
    let icon = weather_icon(
        bundle.current.weather_code,
        IconMode::Unicode,
        bundle.current.is_day,
    )
    .chars()
    .next()
    .unwrap_or('?');
    format!("{temp}°{unit} {icon}")
}

fn temp_display(celsius: f32, units: Units) -> (i32, &'static str) {
    (
        round_temp(convert_temp(celsius, units)),
        if matches!(units, Units::Celsius) {
            "C"
        } else {
            "F"
        },
    )
}

#[cfg(test)]
mod tests {
    use super::atmos_context_line;
    use crate::domain::weather::{
        CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location, Units,
        WeatherCategory,
    };
    use chrono::{NaiveDate, NaiveDateTime, Utc};

    #[test]
    fn atmos_context_ignores_precip_beyond_12h_window() {
        let bundle = sample_bundle_with_precip_at(13, 0.4);
        let context = atmos_context_line(&bundle, Units::Celsius, WeatherCategory::Cloudy);
        assert!(!context.contains("Precipitation expected"));
        assert!(context.contains("overcast skies"));
    }

    #[test]
    fn atmos_context_reports_precip_within_12h_window() {
        let bundle = sample_bundle_with_precip_at(12, 0.4);
        let context = atmos_context_line(&bundle, Units::Celsius, WeatherCategory::Cloudy);
        assert!(context.contains("Precipitation expected"));
    }

    fn sample_bundle_with_precip_at(index: usize, mm: f32) -> ForecastBundle {
        let base = NaiveDateTime::parse_from_str("2026-02-20T15:00", "%Y-%m-%dT%H:%M")
            .expect("valid datetime");
        let hourly = (0..24)
            .map(|idx| HourlyForecast {
                time: base + chrono::Duration::hours(i64::from(idx as u16)),
                temperature_2m_c: Some(1.0),
                weather_code: Some(3),
                is_day: Some(true),
                relative_humidity_2m: Some(50.0),
                precipitation_probability: Some(10.0),
                precipitation_mm: Some(if idx == index { mm } else { 0.0 }),
                rain_mm: Some(0.0),
                snowfall_cm: Some(0.0),
                wind_speed_10m: Some(10.0),
                wind_gusts_10m: Some(14.0),
                pressure_msl_hpa: Some(1010.0),
                visibility_m: Some(9_000.0),
                cloud_cover: Some(70.0),
                cloud_cover_low: Some(20.0),
                cloud_cover_mid: Some(30.0),
                cloud_cover_high: Some(40.0),
            })
            .collect::<Vec<_>>();

        ForecastBundle {
            location: Location::from_coords(59.3293, 18.0686),
            current: CurrentConditions {
                temperature_2m_c: 1.0,
                relative_humidity_2m: 50.0,
                apparent_temperature_c: -1.0,
                dew_point_2m_c: -2.0,
                weather_code: 3,
                precipitation_mm: 0.0,
                cloud_cover: 70.0,
                pressure_msl_hpa: 1010.0,
                visibility_m: 9_000.0,
                wind_speed_10m: 10.0,
                wind_gusts_10m: 14.0,
                wind_direction_10m: 270.0,
                is_day: true,
                high_today_c: Some(2.0),
                low_today_c: Some(-3.0),
            },
            hourly,
            daily: vec![DailyForecast {
                date: NaiveDate::from_ymd_opt(2026, 2, 20).expect("valid date"),
                weather_code: Some(3),
                temperature_max_c: Some(2.0),
                temperature_min_c: Some(-3.0),
                sunrise: None,
                sunset: None,
                uv_index_max: Some(1.0),
                precipitation_probability_max: Some(10.0),
                precipitation_sum_mm: Some(0.0),
                rain_sum_mm: Some(0.0),
                snowfall_sum_cm: Some(0.0),
                precipitation_hours: Some(0.0),
                wind_gusts_10m_max: Some(14.0),
                daylight_duration_s: Some(32_000.0),
                sunshine_duration_s: Some(8_000.0),
            }],
            air_quality: None,
            fetched_at: Utc::now(),
        }
    }
}

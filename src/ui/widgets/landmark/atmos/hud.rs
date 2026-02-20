use crate::cli::IconMode;
use chrono::Timelike;

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_icon};
use crate::ui::widgets::landmark::shared::paint_char;

pub(super) fn atmos_context_line(bundle: &ForecastBundle, category: WeatherCategory) -> String {
    if let Some((last_precip_idx, total_precip_mm)) = precip_summary(bundle) {
        return precip_context(bundle, last_precip_idx, total_precip_mm);
    }
    stable_weather_context(bundle, category)
}

pub(super) fn paint_hud_badge(canvas: &mut [Vec<char>], bundle: &ForecastBundle, width: usize) {
    if width < 30 || canvas.len() < 4 {
        return;
    }
    let badge = hud_badge_text(bundle);
    let start_x = width.saturating_sub(badge.chars().count() + 1);
    for (idx, ch) in badge.chars().enumerate() {
        paint_char(canvas, (start_x + idx) as isize, 0, ch, true);
    }
}

fn precip_summary(bundle: &ForecastBundle) -> Option<(usize, f32)> {
    let mut total_precip = 0.0_f32;
    let mut last_precip_idx = None;
    for (idx, hour) in bundle.hourly.iter().take(24).enumerate() {
        if let Some(mm) = hour.precipitation_mm.filter(|mm| *mm > 0.1) {
            total_precip += mm;
            last_precip_idx = Some(idx);
        }
    }
    last_precip_idx.map(|idx| (idx, total_precip))
}

fn precip_context(bundle: &ForecastBundle, last_precip_idx: usize, total_precip_mm: f32) -> String {
    let now_hour = bundle
        .hourly
        .first()
        .map_or(12, |hour| hour.time.hour() as usize);
    let end_hour = (now_hour + last_precip_idx + 1) % 24;
    let has_precip_now = bundle
        .hourly
        .first()
        .and_then(|h| h.precipitation_mm)
        .unwrap_or(0.0)
        > 0.1;
    if has_precip_now {
        format!("Precip clearing by {end_hour:02}:00 · {total_precip_mm:.0}mm expected")
    } else {
        format!("Precipitation expected through {end_hour:02}:00 · {total_precip_mm:.0}mm")
    }
}

fn stable_weather_context(bundle: &ForecastBundle, category: WeatherCategory) -> String {
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
    let temp_c = bundle.current.temperature_2m_c.round() as i32;
    if matches!(category, WeatherCategory::Unknown) {
        return format!("Currently {temp_c}°C");
    }
    format!("Currently {temp_c}°C · overcast skies")
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

fn hud_badge_text(bundle: &ForecastBundle) -> String {
    let temp_c = bundle.current.temperature_2m_c.round() as i32;
    let icon = weather_icon(
        bundle.current.weather_code,
        IconMode::Unicode,
        bundle.current.is_day,
    )
    .chars()
    .next()
    .unwrap_or('?');
    format!("{temp_c}°C {icon}")
}

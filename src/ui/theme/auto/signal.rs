use crate::domain::weather::{
    ForecastBundle, HourlyForecast, WeatherCategory, weather_code_to_category,
};

use super::AutoThemeSignal;

#[derive(Debug, Clone, Copy)]
struct SignalExtremes {
    max_precip_mm_6h: f32,
    max_snow_cm_6h: f32,
    max_cloud_cover_6h: f32,
    thunder_soon: bool,
    strong_wind_soon: bool,
}

pub(super) fn auto_theme_signal(bundle: &ForecastBundle) -> Option<AutoThemeSignal<'_>> {
    let now_local = bundle.hourly.first().map(|hour| hour.time)?;
    let next_6h = &bundle.hourly[..bundle.hourly.len().min(6)];
    let current_category = weather_code_to_category(bundle.current.weather_code);
    let incoming_category = dominant_category(next_6h, bundle.current.weather_code);
    let extremes = signal_extremes(next_6h);

    Some(AutoThemeSignal {
        now_local,
        sunrise_today: bundle.daily.first().and_then(|day| day.sunrise),
        sunset_today: bundle.daily.first().and_then(|day| day.sunset),
        current_category,
        current_cloud_cover: bundle.current.cloud_cover,
        current_visibility_m: bundle.current.visibility_m,
        current_precip_mm: bundle.current.precipitation_mm.max(0.0),
        next_6h,
        incoming_category,
        max_precip_mm_6h: extremes.max_precip_mm_6h,
        max_snow_cm_6h: extremes.max_snow_cm_6h,
        max_cloud_cover_6h: extremes.max_cloud_cover_6h,
        thunder_soon: extremes.thunder_soon,
        strong_wind_soon: extremes.strong_wind_soon,
        clearing_soon: is_clearing_soon(current_category, incoming_category),
    })
}

pub(super) fn rain_now_or_soon(signal: &AutoThemeSignal<'_>) -> bool {
    signal.current_precip_mm > 0.05
        || signal.current_category == WeatherCategory::Rain
        || signal.incoming_category == WeatherCategory::Rain
        || signal
            .next_6h
            .iter()
            .any(|hour| hour.precipitation_mm.unwrap_or(0.0) > 0.05)
}

pub(super) fn snow_now_or_soon(signal: &AutoThemeSignal<'_>) -> bool {
    signal.current_category == WeatherCategory::Snow
        || signal.incoming_category == WeatherCategory::Snow
        || signal.max_snow_cm_6h > 0.0
}

pub(super) fn fog_now(signal: &AutoThemeSignal<'_>) -> bool {
    signal.current_visibility_m < 3000.0 || signal.current_category == WeatherCategory::Fog
}

fn signal_extremes(next_6h: &[HourlyForecast]) -> SignalExtremes {
    SignalExtremes {
        max_precip_mm_6h: next_6h
            .iter()
            .filter_map(|hour| hour.precipitation_mm)
            .fold(0.0, f32::max),
        max_snow_cm_6h: next_6h
            .iter()
            .filter_map(|hour| hour.snowfall_cm)
            .fold(0.0, f32::max),
        max_cloud_cover_6h: next_6h
            .iter()
            .filter_map(|hour| hour.cloud_cover)
            .fold(0.0, f32::max),
        thunder_soon: next_6h.iter().any(hour_has_thunder),
        strong_wind_soon: next_6h
            .iter()
            .filter_map(|hour| hour.wind_gusts_10m)
            .any(|gust| gust >= 60.0),
    }
}

fn dominant_category(next_6h: &[HourlyForecast], fallback_code: u8) -> WeatherCategory {
    let mut counts = [0usize; 7];
    for hour in next_6h {
        let category = weather_code_to_category(hour.weather_code.unwrap_or(fallback_code));
        counts[category_index(category)] += 1;
    }

    counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| **count)
        .map_or_else(
            || weather_code_to_category(fallback_code),
            |(idx, _)| category_from_index(idx),
        )
}

fn is_clearing_soon(current_category: WeatherCategory, incoming_category: WeatherCategory) -> bool {
    matches!(
        current_category,
        WeatherCategory::Rain | WeatherCategory::Snow | WeatherCategory::Fog
    ) && matches!(
        incoming_category,
        WeatherCategory::Clear | WeatherCategory::Cloudy
    )
}

fn hour_has_thunder(hour: &HourlyForecast) -> bool {
    hour.weather_code
        .is_some_and(|code| (95..=99).contains(&usize::from(code)))
}

fn category_index(category: WeatherCategory) -> usize {
    match category {
        WeatherCategory::Clear => 0,
        WeatherCategory::Cloudy => 1,
        WeatherCategory::Rain => 2,
        WeatherCategory::Snow => 3,
        WeatherCategory::Fog => 4,
        WeatherCategory::Thunder => 5,
        WeatherCategory::Unknown => 6,
    }
}

fn category_from_index(index: usize) -> WeatherCategory {
    match index {
        0 => WeatherCategory::Clear,
        1 => WeatherCategory::Cloudy,
        2 => WeatherCategory::Rain,
        3 => WeatherCategory::Snow,
        4 => WeatherCategory::Fog,
        5 => WeatherCategory::Thunder,
        _ => WeatherCategory::Unknown,
    }
}

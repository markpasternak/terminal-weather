use super::*;
use crate::cli::ThemeArg;
use crate::domain::weather::{CurrentConditions, Location, WeatherCategory};
use crate::ui::theme::{ColorCapability, theme_for};
use chrono::{NaiveDate, Utc};

fn sample_day(precip_mm: f32, gust_kmh: f32, uv: f32) -> DailyForecast {
    DailyForecast {
        date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("date"),
        weather_code: Some(3),
        temperature_max_c: Some(8.0),
        temperature_min_c: Some(1.0),
        sunrise: None,
        sunset: None,
        uv_index_max: Some(uv),
        precipitation_probability_max: Some(50.0),
        precipitation_sum_mm: Some(precip_mm),
        rain_sum_mm: Some(precip_mm),
        snowfall_sum_cm: Some(0.0),
        precipitation_hours: Some(1.0),
        wind_gusts_10m_max: Some(gust_kmh),
        daylight_duration_s: Some(36_000.0),
        sunshine_duration_s: Some(18_000.0),
    }
}

fn sample_bundle_with_daily(daily: Vec<DailyForecast>) -> ForecastBundle {
    ForecastBundle {
        location: Location::from_coords(59.33, 18.07),
        current: CurrentConditions {
            temperature_2m_c: 5.0,
            relative_humidity_2m: 60.0,
            apparent_temperature_c: 3.0,
            dew_point_2m_c: 1.0,
            weather_code: 3,
            precipitation_mm: 0.0,
            cloud_cover: 50.0,
            pressure_msl_hpa: 1010.0,
            visibility_m: 9000.0,
            wind_speed_10m: 8.0,
            wind_gusts_10m: 12.0,
            wind_direction_10m: 270.0,
            is_day: true,
            high_today_c: Some(8.0),
            low_today_c: Some(1.0),
        },
        hourly: vec![],
        daily,
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

fn test_theme() -> crate::ui::theme::Theme {
    theme_for(
        WeatherCategory::Clear,
        true,
        ColorCapability::Basic16,
        ThemeArg::Auto,
    )
}

mod formatting;
mod profiles;
mod rendering;

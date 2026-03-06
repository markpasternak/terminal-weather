use super::super::AmbientSkyLifeContext;
use crate::domain::weather::WeatherCategory;
use crate::ui::animation::{MotionMode, SeededMotion, UiMotionContext};

pub(super) fn blank_canvas(width: usize, height: usize) -> Vec<Vec<char>> {
    vec![vec![' '; width]; height]
}

pub(super) fn ambient_marks(canvas: &[Vec<char>]) -> usize {
    canvas
        .iter()
        .flatten()
        .filter(|ch| matches!(**ch, '<' | '^' | '~' | '>'))
        .count()
}

pub(super) fn base_ctx() -> AmbientSkyLifeContext {
    AmbientSkyLifeContext {
        category: WeatherCategory::Clear,
        is_day: true,
        cloud_pct: 30.0,
        wind_speed: 10.0,
        phase: 24,
        animate: true,
        horizon_y: 10,
        width: 40,
        seed: SeededMotion::new(17),
        elapsed_seconds: 1.4,
    }
}

pub(super) fn motion() -> UiMotionContext {
    UiMotionContext {
        elapsed_seconds: 1.0,
        dt_seconds: 0.05,
        frame_index: 8,
        motion_mode: MotionMode::Cinematic,
        seed: SeededMotion::new(99),
        weather_profile: None,
        transition_progress: None,
        animate: true,
    }
}

pub(super) fn seed(label: &str) -> SeededMotion {
    SeededMotion::new(crate::ui::animation::stable_hash(&label))
}

pub(super) fn bundle_for_category(
    weather_code: u8,
    is_day: bool,
) -> crate::domain::weather::ForecastBundle {
    use crate::domain::weather::Location;
    use chrono::NaiveDateTime;

    let base = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").expect("time");
    crate::domain::weather::ForecastBundle {
        location: Location::from_coords(59.33, 18.07),
        current: sample_current(weather_code, is_day),
        hourly: sample_hourly(base, weather_code, is_day),
        daily: vec![crate::test_support::sample_daily()],
        air_quality: None,
        fetched_at: chrono::Utc::now(),
    }
}

fn sample_current(weather_code: u8, is_day: bool) -> crate::domain::weather::CurrentConditions {
    crate::domain::weather::CurrentConditions {
        temperature_2m_c: 5.0,
        relative_humidity_2m: 60.0,
        apparent_temperature_c: 3.0,
        dew_point_2m_c: 1.0,
        weather_code,
        precipitation_mm: if weather_code > 10 { 2.0 } else { 0.0 },
        cloud_cover: 50.0,
        pressure_msl_hpa: 1010.0,
        visibility_m: 9000.0,
        wind_speed_10m: 8.0,
        wind_gusts_10m: 12.0,
        wind_direction_10m: 270.0,
        is_day,
        high_today_c: Some(8.0),
        low_today_c: Some(1.0),
    }
}

fn sample_hourly(
    base: chrono::NaiveDateTime,
    weather_code: u8,
    is_day: bool,
) -> Vec<crate::domain::weather::HourlyForecast> {
    (0..24)
        .map(|i| crate::domain::weather::HourlyForecast {
            time: base + chrono::Duration::hours(i64::from(i as u16)),
            temperature_2m_c: Some(4.0 + i as f32 * 0.1),
            weather_code: Some(weather_code),
            is_day: Some(is_day),
            relative_humidity_2m: Some(60.0),
            precipitation_probability: Some(30.0),
            precipitation_mm: Some(0.5),
            rain_mm: Some(0.3),
            snowfall_cm: Some(0.0),
            wind_speed_10m: Some(8.0),
            wind_gusts_10m: Some(12.0),
            pressure_msl_hpa: Some(1010.0),
            visibility_m: Some(9000.0),
            cloud_cover: Some(50.0),
            cloud_cover_low: Some(10.0),
            cloud_cover_mid: Some(20.0),
            cloud_cover_high: Some(30.0),
        })
        .collect()
}

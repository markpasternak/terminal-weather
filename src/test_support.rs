use chrono::{NaiveDate, NaiveDateTime, Utc};

use crate::{
    cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
    domain::weather::{CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location},
};

fn parse_time(value: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").expect("valid time fixture")
}

pub(crate) fn hero_test_cli() -> Cli {
    Cli {
        city: Some("Stockholm".to_string()),
        units: UnitsArg::Celsius,
        fps: 30,
        no_animation: true,
        reduced_motion: false,
        no_flash: true,
        ascii_icons: false,
        emoji_icons: false,
        color: ColorArg::Auto,
        no_color: false,
        hourly_view: None,
        theme: ThemeArg::Auto,
        hero_visual: HeroVisualArg::AtmosCanvas,
        country_code: None,
        lat: None,
        lon: None,
        forecast_url: None,
        air_quality_url: None,
        refresh_interval: 600,
        demo: false,
        one_shot: false,
    }
}

pub(crate) fn state_test_cli() -> Cli {
    Cli {
        city: None,
        units: UnitsArg::Celsius,
        fps: 30,
        no_animation: true,
        reduced_motion: false,
        no_flash: true,
        ascii_icons: false,
        emoji_icons: false,
        color: ColorArg::Auto,
        no_color: false,
        hourly_view: None,
        theme: ThemeArg::Auto,
        hero_visual: HeroVisualArg::AtmosCanvas,
        country_code: None,
        lat: None,
        lon: None,
        forecast_url: None,
        air_quality_url: None,
        refresh_interval: 600,
        demo: false,
        one_shot: false,
    }
}

pub(crate) fn settings_default_test_cli() -> Cli {
    Cli {
        city: None,
        units: UnitsArg::Celsius,
        fps: 30,
        no_animation: false,
        reduced_motion: false,
        no_flash: false,
        ascii_icons: false,
        emoji_icons: false,
        color: ColorArg::Auto,
        no_color: false,
        hourly_view: None,
        theme: ThemeArg::Auto,
        hero_visual: HeroVisualArg::AtmosCanvas,
        country_code: None,
        lat: None,
        lon: None,
        forecast_url: None,
        air_quality_url: None,
        refresh_interval: 600,
        demo: false,
        one_shot: false,
    }
}

pub(crate) fn stockholm_location() -> Location {
    Location {
        name: "Stockholm".to_string(),
        latitude: 59.3293,
        longitude: 18.0686,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
        population: None,
    }
}

pub(crate) fn sample_current() -> CurrentConditions {
    CurrentConditions {
        temperature_2m_c: 7.0,
        relative_humidity_2m: 72.0,
        apparent_temperature_c: 5.0,
        dew_point_2m_c: 2.0,
        weather_code: 3,
        precipitation_mm: 0.0,
        cloud_cover: 40.0,
        pressure_msl_hpa: 1008.0,
        visibility_m: 10_000.0,
        wind_speed_10m: 10.0,
        wind_gusts_10m: 15.0,
        wind_direction_10m: 180.0,
        is_day: true,
        high_today_c: Some(8.0),
        low_today_c: Some(1.0),
    }
}

pub(crate) fn sample_hourly() -> HourlyForecast {
    HourlyForecast {
        time: parse_time("2026-02-12T10:00"),
        temperature_2m_c: Some(7.0),
        weather_code: Some(3),
        is_day: Some(true),
        relative_humidity_2m: Some(72.0),
        precipitation_probability: Some(35.0),
        precipitation_mm: Some(0.0),
        rain_mm: Some(0.0),
        snowfall_cm: Some(0.0),
        wind_speed_10m: Some(10.0),
        wind_gusts_10m: Some(15.0),
        pressure_msl_hpa: Some(1008.0),
        visibility_m: Some(10_000.0),
        cloud_cover: Some(40.0),
        cloud_cover_low: Some(20.0),
        cloud_cover_mid: Some(30.0),
        cloud_cover_high: Some(35.0),
    }
}

pub(crate) fn sample_daily() -> DailyForecast {
    DailyForecast {
        date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("valid date fixture"),
        weather_code: Some(3),
        temperature_max_c: Some(8.0),
        temperature_min_c: Some(1.0),
        sunrise: None,
        sunset: None,
        uv_index_max: Some(2.0),
        precipitation_probability_max: Some(35.0),
        precipitation_sum_mm: Some(0.0),
        rain_sum_mm: Some(0.0),
        snowfall_sum_cm: Some(0.0),
        precipitation_hours: Some(0.0),
        wind_gusts_10m_max: Some(15.0),
        daylight_duration_s: Some(32_000.0),
        sunshine_duration_s: Some(18_000.0),
    }
}

pub(crate) fn sample_bundle() -> ForecastBundle {
    ForecastBundle {
        location: stockholm_location(),
        current: sample_current(),
        hourly: vec![sample_hourly()],
        daily: vec![sample_daily()],
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

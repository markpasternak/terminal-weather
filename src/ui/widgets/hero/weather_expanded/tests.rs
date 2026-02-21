use super::{
    expanded_fetch_context, last_updated_label, trends::next_precip_summary,
    trends::pressure_span_summary,
};
use crate::{
    app::state::AppState,
    cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
    domain::weather::{CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location},
    resilience::freshness::FreshnessState,
};
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};

#[test]
fn next_precip_summary_covers_now_in_nh_and_none() {
    let now = vec![hour(0, Some(0.4)), hour(1, None), hour(2, None)];
    assert_eq!(next_precip_summary(&now), "now (0.4mm)");

    let later = vec![hour(0, Some(0.0)), hour(1, Some(0.1)), hour(2, Some(0.3))];
    assert_eq!(next_precip_summary(&later), "in 2h (0.3mm)");

    let dry = vec![hour(0, Some(0.0)), hour(1, None), hour(2, Some(0.1))];
    assert_eq!(next_precip_summary(&dry), "none in 12h");
}

#[test]
fn next_precip_summary_includes_12h_boundary() {
    let mut hourly = (0..13).map(|idx| hour(idx, Some(0.0))).collect::<Vec<_>>();
    hourly[12].precipitation_mm = Some(0.4);
    assert_eq!(next_precip_summary(&hourly), "in 12h (0.4mm)");
}

#[test]
fn pressure_span_summary_handles_empty_and_non_empty() {
    assert_eq!(pressure_span_summary(&[]), "--");
    assert_eq!(
        pressure_span_summary(&[1008.2, 1012.9, 1010.0]),
        "1008..1013hPa"
    );
}

#[test]
fn last_updated_label_without_success_uses_placeholder() {
    let state = AppState::new(&test_cli());
    let weather = sample_bundle();
    let label = last_updated_label(&state, &weather);
    assert!(label.starts_with("Last updated --:-- local"));
    assert!(label.ends_with("City TZ Europe/Stockholm"));
}

#[test]
fn last_updated_label_includes_age_and_timezone() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.last_success = Some(Utc::now() - Duration::minutes(4));
    let weather = sample_bundle();
    let label = last_updated_label(&state, &weather);
    assert!(label.contains("local ("));
    assert!(label.contains("City TZ Europe/Stockholm"));
}

#[test]
fn expanded_fetch_context_is_suppressed_when_fresh() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Fresh;
    state.last_error = Some("transient error".to_string());
    assert!(expanded_fetch_context(&state).is_none());
}

#[test]
fn expanded_fetch_context_truncates_long_multiline_errors() {
    let mut state = AppState::new(&test_cli());
    state.refresh_meta.state = FreshnessState::Offline;
    state.last_error = Some(format!(
        "{}\n{}",
        "x".repeat(120),
        "this second line should not appear"
    ));

    let line = expanded_fetch_context(&state).expect("fetch context line");
    assert!(line.starts_with("Last fetch failed: "));
    assert!(!line.contains("second line"));
    assert!(line.contains('…'));
}

fn test_cli() -> Cli {
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

fn sample_bundle() -> ForecastBundle {
    ForecastBundle {
        location: sample_location(),
        current: sample_current(),
        hourly: vec![sample_hourly()],
        daily: vec![sample_daily()],
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

fn sample_location() -> Location {
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

fn sample_current() -> CurrentConditions {
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

fn sample_hourly() -> HourlyForecast {
    HourlyForecast {
        time: NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M")
            .expect("valid time"),
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

fn sample_daily() -> DailyForecast {
    DailyForecast {
        date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("valid date"),
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

fn dt(hour: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 2, 20)
        .expect("valid date")
        .and_hms_opt(hour, 0, 0)
        .expect("valid time")
}

fn hour(hour: u32, precip_mm: Option<f32>) -> HourlyForecast {
    HourlyForecast {
        time: dt(hour),
        temperature_2m_c: None,
        weather_code: None,
        is_day: None,
        relative_humidity_2m: None,
        precipitation_probability: None,
        precipitation_mm: precip_mm,
        rain_mm: None,
        snowfall_cm: None,
        wind_speed_10m: None,
        wind_gusts_10m: None,
        pressure_msl_hpa: None,
        visibility_m: None,
        cloud_cover: None,
        cloud_cover_low: None,
        cloud_cover_mid: None,
        cloud_cover_high: None,
    }
}

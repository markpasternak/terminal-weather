use super::daypart::daypart_visibility;
use super::table::{build_optional_date_row, metric_row_specs, sanitize_precip_mm};
use super::*;
use crate::{
    cli::ThemeArg,
    domain::weather::{HourlyForecast, HourlyViewMode, WeatherCategory},
    ui::theme::{ColorCapability, theme_for},
};
use chrono::{NaiveDate, NaiveDateTime};
use ratatui::layout::Rect;

#[test]
fn width_below_70_forces_table() {
    let area = Rect::new(0, 0, 68, 12);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, area),
        HourlyViewMode::Table
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, area),
        HourlyViewMode::Table
    );
}

#[test]
fn chart_mode_requires_more_height_than_hybrid() {
    let hybrid_ok = Rect::new(0, 0, 90, 10);
    let chart_too_short = Rect::new(0, 0, 90, 9);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, hybrid_ok),
        HourlyViewMode::Hybrid
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, chart_too_short),
        HourlyViewMode::Table
    );
}

#[test]
fn sanitize_precip_non_negative() {
    assert!((sanitize_precip_mm(-1.0) - 0.0).abs() < f32::EPSILON);
    assert!((sanitize_precip_mm(0.0) - 0.0).abs() < f32::EPSILON);
    assert!((sanitize_precip_mm(1.2) - 1.2).abs() < f32::EPSILON);
}

#[test]
fn optional_metric_rows_keep_height_thresholds() {
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let count_for = |height: u16| {
        metric_row_specs(theme)
            .iter()
            .filter(|(min_height, _, _, _)| *min_height <= height)
            .count()
    };

    assert_eq!(count_for(4), 0);
    assert_eq!(count_for(5), 1);
    assert_eq!(count_for(6), 2);
    assert_eq!(count_for(9), 5);
    assert_eq!(count_for(12), 8);
}

#[test]
fn date_row_inserts_for_day_change_or_offset() {
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let hours = [
        sample_hour(dt(2026, 2, 20, 9)),
        sample_hour(dt(2026, 2, 20, 10)),
        sample_hour(dt(2026, 2, 21, 0)),
    ];
    let same_day = vec![&hours[0], &hours[1]];
    let crosses_day = vec![&hours[1], &hours[2]];

    assert!(build_optional_date_row(&same_day, 0, theme).is_none());
    assert!(build_optional_date_row(&crosses_day, 0, theme).is_some());
    assert!(build_optional_date_row(&same_day, 1, theme).is_some());
}

#[test]
fn daypart_visibility_thresholds_match_layout_contract() {
    assert_eq!(daypart_visibility(3), (false, false, false));
    assert_eq!(daypart_visibility(4), (false, false, true));
    assert_eq!(daypart_visibility(5), (false, true, true));
    assert_eq!(daypart_visibility(6), (true, true, true));
}

fn dt(year: i32, month: u32, day: u32, hour: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("valid date")
        .and_hms_opt(hour, 0, 0)
        .expect("valid time")
}

fn sample_hour(time: NaiveDateTime) -> HourlyForecast {
    HourlyForecast {
        time,
        temperature_2m_c: Some(1.0),
        weather_code: Some(3),
        is_day: Some(true),
        relative_humidity_2m: Some(75.0),
        precipitation_probability: Some(10.0),
        precipitation_mm: Some(0.2),
        rain_mm: Some(0.2),
        snowfall_cm: Some(0.0),
        wind_speed_10m: Some(12.0),
        wind_gusts_10m: Some(18.0),
        pressure_msl_hpa: Some(1009.0),
        visibility_m: Some(8000.0),
        cloud_cover: Some(60.0),
        cloud_cover_low: Some(20.0),
        cloud_cover_mid: Some(25.0),
        cloud_cover_high: Some(15.0),
    }
}

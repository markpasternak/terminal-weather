use super::super::test_support::{
    assert_fetch_context_suppressed_when_fresh,
    assert_fetch_context_truncates_long_multiline_errors, assert_last_updated_placeholder,
    sample_bundle, test_cli,
};
use super::{
    expanded_fetch_context, last_updated_label, trends::next_precip_summary,
    trends::pressure_span_summary,
};
use crate::{app::state::AppState, domain::weather::HourlyForecast};
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
    assert_last_updated_placeholder(last_updated_label, "Last updated --:-- local");
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
    assert_fetch_context_suppressed_when_fresh(expanded_fetch_context);
}

#[test]
fn expanded_fetch_context_truncates_long_multiline_errors() {
    assert_fetch_context_truncates_long_multiline_errors(expanded_fetch_context);
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

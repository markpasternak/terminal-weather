use super::trends::{next_precip_summary, pressure_span_summary};
use crate::domain::weather::HourlyForecast;
use chrono::{NaiveDate, NaiveDateTime};

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
fn pressure_span_summary_handles_empty_and_non_empty() {
    assert_eq!(pressure_span_summary(&[]), "--");
    assert_eq!(
        pressure_span_summary(&[1008.2, 1012.9, 1010.0]),
        "1008..1013hPa"
    );
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

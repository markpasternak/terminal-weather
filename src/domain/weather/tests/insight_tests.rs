use super::*;
use crate::resilience::freshness::FreshnessState;

fn base_time() -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::parse_from_str("2026-02-12T08:00", "%Y-%m-%dT%H:%M").unwrap()
}

fn time_plus_hours(hours: i64) -> chrono::NaiveDateTime {
    base_time() + chrono::Duration::hours(hours)
}

fn fresh_meta() -> RefreshMetadata {
    RefreshMetadata {
        state: FreshnessState::Fresh,
        ..RefreshMetadata::default()
    }
}

fn hourly_precip_fixture(time: chrono::NaiveDateTime, precip_mm: f32) -> HourlyForecast {
    HourlyForecast {
        time,
        temperature_2m_c: None,
        weather_code: None,
        is_day: None,
        relative_humidity_2m: None,
        precipitation_probability: None,
        precipitation_mm: Some(precip_mm),
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

#[test]
fn summarize_precip_window_includes_12h_boundary() {
    let base = time_plus_hours(7);
    let hourly = (0..13)
        .map(|idx| {
            hourly_precip_fixture(
                base + chrono::Duration::hours(i64::from(idx)),
                if idx == 12 { 0.4 } else { 0.0 },
            )
        })
        .collect::<Vec<_>>();

    let summary = summarize_precip_window(
        &hourly,
        PRECIP_NEAR_TERM_HOURS,
        PRECIP_SIGNIFICANT_THRESHOLD_MM,
    )
    .expect("precip summary");
    assert_eq!(summary.first_idx, 12);
    assert!((summary.first_amount_mm - 0.4).abs() < 0.001);
}

#[test]
fn summarize_precip_window_guard_cases() {
    assert!(summarize_precip_window(&[], 0, 0.1).is_none());
    assert!(summarize_precip_window(&[], 12, -1.0).is_none());
    assert!(summarize_precip_window(&[], 12, 0.1).is_none());
}

#[test]
fn precip_window_has_precip_now_when_first_hour_has_precip() {
    let hourly = vec![hourly_precip_fixture(base_time(), 1.5)];
    let summary = summarize_precip_window(&hourly, 12, 0.1).expect("should find precip");
    assert!(summary.has_precip_now());
}

#[test]
fn derive_nowcast_insight_returns_rain_action_when_precip_signal_is_high() {
    let base = base_time();
    let mut bundle = minimal_bundle(Some(8.0), Some(1.0));
    bundle.hourly = vec![
        sample_hour(base, 5.0, 3, 10.0, 0.0, 12.0, 9_000.0),
        sample_hour(
            base + chrono::Duration::hours(1),
            5.0,
            61,
            75.0,
            0.4,
            14.0,
            8_000.0,
        ),
    ];
    let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
    assert_eq!(insight.action, ActionCue::CarryUmbrella);
    assert!(insight.action_text.contains("precipitation gear"));
}

#[test]
fn derive_nowcast_insight_ignores_distant_light_precip_for_now_action() {
    let base = base_time();
    let mut bundle = minimal_bundle(Some(8.0), Some(1.0));
    bundle.hourly = (0..10)
        .map(|idx| {
            let (prob, mm) = if idx == 8 { (100.0, 0.3) } else { (10.0, 0.0) };
            sample_hour(
                base + chrono::Duration::hours(i64::from(idx)),
                5.0,
                3,
                prob,
                mm,
                10.0,
                9_000.0,
            )
        })
        .collect();
    let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
    assert_eq!(insight.action, ActionCue::Comfortable);
}

#[test]
fn derive_nowcast_insight_uses_winter_action_for_snow_signal() {
    let base = base_time();
    let mut bundle = minimal_bundle(Some(0.0), Some(-5.0));
    bundle.current.temperature_2m_c = -3.0;
    bundle.current.weather_code = 71;
    bundle.hourly = vec![
        sample_hour(base, -3.0, 3, 10.0, 0.0, 10.0, 9_000.0),
        sample_hour(
            base + chrono::Duration::hours(1),
            -2.5,
            71,
            90.0,
            0.6,
            10.0,
            8_000.0,
        ),
    ];
    let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
    assert_eq!(insight.action, ActionCue::WinterTraction);
    assert!(insight.action_text.contains("winter traction"));
}

#[test]
fn derive_nowcast_insight_uses_rain_action_for_nonfreezing_thunder() {
    let base = base_time();
    let mut bundle = minimal_bundle(Some(11.0), Some(6.0));
    bundle.current.temperature_2m_c = 7.0;
    bundle.current.weather_code = 95;
    bundle.hourly = vec![
        sample_hour(base, 7.0, 95, 85.0, 0.8, 20.0, 8_000.0),
        sample_hour(
            base + chrono::Duration::hours(1),
            7.2,
            95,
            88.0,
            0.9,
            21.0,
            8_000.0,
        ),
    ];
    bundle.hourly[0].snowfall_cm = Some(0.2);
    bundle.hourly[1].snowfall_cm = Some(0.2);
    let insight = derive_nowcast_insight(&bundle, Units::Celsius, &fresh_meta());
    assert_eq!(insight.action, ActionCue::CarryUmbrella);
}

#[test]
fn next_notable_change_prefers_precip_start_over_later_changes() {
    let base = base_time();
    let hourly = vec![
        sample_hour(base, 5.0, 3, 5.0, 0.0, 10.0, 9_000.0),
        sample_hour(
            base + chrono::Duration::hours(1),
            5.2,
            61,
            75.0,
            0.3,
            11.0,
            8_500.0,
        ),
        sample_hour(
            base + chrono::Duration::hours(2),
            0.0,
            61,
            60.0,
            0.5,
            40.0,
            8_000.0,
        ),
    ];

    let change = next_notable_change(&hourly, Units::Celsius).expect("notable change");
    assert_eq!(change.kind, ChangeKind::PrecipStart);
    assert_eq!(change.hours_from_now, 1);
}

#[test]
fn derive_nowcast_insight_confidence_drops_to_low_when_offline() {
    let base = base_time();
    let mut bundle = minimal_bundle(Some(8.0), Some(1.0));
    bundle.hourly = vec![sample_hour(base, 5.0, 3, 10.0, 0.0, 10.0, 9_000.0)];
    let refresh_meta = RefreshMetadata {
        state: FreshnessState::Offline,
        ..RefreshMetadata::default()
    };

    let insight = derive_nowcast_insight(&bundle, Units::Celsius, &refresh_meta);
    assert_eq!(insight.confidence, InsightConfidence::Low);
}

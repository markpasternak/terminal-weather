use super::*;
use crate::cli::IconMode;

#[test]
fn freezing_drizzle_codes_have_labels() {
    assert_eq!(weather_label(56), "Light freezing drizzle");
    assert_eq!(weather_label(57), "Dense freezing drizzle");
}

#[test]
fn clear_conditions_respect_day_night_for_labels_and_icons() {
    assert_eq!(weather_label_for_time(0, true), "Clear sky");
    assert_eq!(weather_label_for_time(0, false), "Clear night");
    assert_eq!(weather_icon(0, IconMode::Ascii, true), "SUN");
    assert_eq!(weather_icon(0, IconMode::Ascii, false), "MON");
}

#[test]
fn daypart_bucket_boundaries_are_correct() {
    let parse = |s: &str| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap();
    assert_eq!(daypart_for_time(parse("2026-02-12T05:59")), Daypart::Night);
    assert_eq!(
        daypart_for_time(parse("2026-02-12T06:00")),
        Daypart::Morning
    );
    assert_eq!(
        daypart_for_time(parse("2026-02-12T11:59")),
        Daypart::Morning
    );
    assert_eq!(daypart_for_time(parse("2026-02-12T12:00")), Daypart::Noon);
    assert_eq!(daypart_for_time(parse("2026-02-12T17:59")), Daypart::Noon);
    assert_eq!(
        daypart_for_time(parse("2026-02-12T18:00")),
        Daypart::Evening
    );
    assert_eq!(
        daypart_for_time(parse("2026-02-12T23:59")),
        Daypart::Evening
    );
    assert_eq!(daypart_for_time(parse("2026-02-13T00:00")), Daypart::Night);
}

#[test]
fn daypart_summary_aggregates_expected_fields() {
    let start =
        chrono::NaiveDateTime::parse_from_str("2026-02-12T06:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = vec![
        sample_hour(start, 5.0, 61, 30.0, 0.2, 10.0, 10_000.0),
        sample_hour(
            start + chrono::Duration::hours(1),
            7.0,
            61,
            40.0,
            0.4,
            12.0,
            8_000.0,
        ),
        sample_hour(
            start + chrono::Duration::hours(2),
            6.0,
            3,
            20.0,
            0.1,
            11.0,
            9_000.0,
        ),
    ];

    let summaries = summarize_dayparts(&hourly, 0, 1);
    let morning = summaries
        .iter()
        .find(|s| s.daypart == Daypart::Morning)
        .expect("morning summary");

    assert_eq!(morning.sample_count, 3);
    assert_eq!(morning.weather_code, 61);
    assert_eq!(morning.temp_min_c, Some(5.0));
    assert_eq!(morning.temp_max_c, Some(7.0));
    assert_eq!(morning.wind_min_kmh, Some(10.0));
    assert_eq!(morning.wind_max_kmh, Some(12.0));
    assert!((morning.precip_sum_mm - 0.7).abs() < 0.001);
    assert_eq!(morning.precip_probability_max, Some(40.0));
    assert_eq!(morning.visibility_median_m, Some(9_000.0));
}

#[test]
fn summarize_dayparts_returns_empty_for_zero_max_days() {
    let base = chrono::NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = vec![sample_hour(base, 5.0, 3, 30.0, 0.0, 10.0, 9000.0)];
    let result = summarize_dayparts(&hourly, 3, 0);
    assert!(result.is_empty());
}

#[test]
fn summarize_dayparts_returns_empty_for_empty_hourly() {
    let result = summarize_dayparts(&[], 3, 3);
    assert!(result.is_empty());
}

#[test]
fn daypart_aggregation_handles_all_none_fields() {
    let base = chrono::NaiveDateTime::parse_from_str("2026-02-12T06:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = vec![HourlyForecast {
        time: base,
        temperature_2m_c: None,
        weather_code: None,
        is_day: None,
        relative_humidity_2m: None,
        precipitation_probability: None,
        precipitation_mm: None,
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
    }];
    let summaries = summarize_dayparts(&hourly, 0, 1);
    let morning = summaries.iter().find(|s| s.daypart == Daypart::Morning);
    if let Some(morning) = morning {
        assert!(morning.temp_min_c.is_none());
        assert!(morning.temp_max_c.is_none());
        assert!(morning.wind_min_kmh.is_none());
    }
}

#[test]
fn high_low_calculation() {
    let bundle = minimal_bundle(Some(25.4), Some(10.6));

    assert_eq!(bundle.high_low(Units::Celsius), Some((25, 11)));
    assert_eq!(bundle.high_low(Units::Fahrenheit), Some((78, 51)));

    let bundle = minimal_bundle(None, Some(10.6));
    assert_eq!(bundle.high_low(Units::Celsius), None);

    let bundle = minimal_bundle(Some(25.4), None);
    assert_eq!(bundle.high_low(Units::Celsius), None);
}

#[test]
fn weather_code_to_particle_all_categories() {
    assert_eq!(weather_code_to_particle(0), ParticleKind::None);
    assert_eq!(weather_code_to_particle(3), ParticleKind::None);
    assert_eq!(weather_code_to_particle(61), ParticleKind::Rain);
    assert_eq!(weather_code_to_particle(71), ParticleKind::Snow);
    assert_eq!(weather_code_to_particle(45), ParticleKind::Fog);
    assert_eq!(weather_code_to_particle(95), ParticleKind::Thunder);
}

#[test]
fn weather_label_for_time_code_one_day_night() {
    assert!(weather_label_for_time(1, true).contains("Mainly clear"));
    assert!(weather_label_for_time(1, false).contains("Mainly clear night"));
}

#[test]
fn weather_icon_all_modes() {
    let emoji = weather_icon(0, IconMode::Emoji, true);
    assert!(!emoji.is_empty());

    let uni = weather_icon(0, IconMode::Unicode, true);
    assert!(!uni.is_empty());

    let day = weather_icon(0, IconMode::Unicode, true);
    let night = weather_icon(0, IconMode::Unicode, false);
    assert_ne!(day, night);
}

#[test]
fn forecast_bundle_current_helpers_return_correct_values() {
    let bundle = minimal_bundle(Some(8.0), Some(1.0));
    assert_eq!(bundle.current_weather_code(), 0);
    assert_eq!(bundle.current_temp(Units::Celsius), 20);
    assert_eq!(bundle.current_temp(Units::Fahrenheit), 68);
}

#[test]
fn location_display_name_name_only() {
    let loc = Location {
        name: "Unknown City".to_string(),
        latitude: 0.0,
        longitude: 0.0,
        country: None,
        admin1: None,
        timezone: None,
        population: None,
    };
    assert_eq!(loc.display_name(), "Unknown City");
}

#[test]
fn parse_date_success_and_failure() {
    assert!(parse_date("2026-02-12").is_some());
    assert!(parse_date("not-a-date").is_none());
}

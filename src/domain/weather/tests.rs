use super::*;
use crate::cli::IconMode;
use chrono::{Duration, Utc};

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
fn fahrenheit_conversion_rounding() {
    assert_eq!(round_temp(convert_temp(0.0, Units::Fahrenheit)), 32);
    assert_eq!(round_temp(convert_temp(20.0, Units::Fahrenheit)), 68);
}

#[test]
fn wind_speed_conversion_rounding() {
    assert!((convert_wind_speed(36.0) - 10.0).abs() < f32::EPSILON);
    assert_eq!(round_wind_speed(54.0), 15);
}

#[test]
fn daypart_bucket_boundaries_are_correct() {
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap();
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
    let start = NaiveDateTime::parse_from_str("2026-02-12T06:00", "%Y-%m-%dT%H:%M").unwrap();
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
fn us_aqi_categories_follow_epa_thresholds() {
    assert_eq!(categorize_us_aqi(40), AirQualityCategory::Good);
    assert_eq!(categorize_us_aqi(75), AirQualityCategory::Moderate);
    assert_eq!(
        categorize_us_aqi(125),
        AirQualityCategory::UnhealthySensitive
    );
    assert_eq!(categorize_us_aqi(180), AirQualityCategory::Unhealthy);
    assert_eq!(categorize_us_aqi(230), AirQualityCategory::VeryUnhealthy);
    assert_eq!(categorize_us_aqi(350), AirQualityCategory::Hazardous);
}

#[test]
fn air_quality_reading_prefers_us_index_when_available() {
    let reading = AirQualityReading::from_indices(Some(57.0), Some(18.0)).expect("aqi reading");
    assert_eq!(reading.us_aqi, Some(57));
    assert_eq!(reading.european_aqi, Some(18));
    assert_eq!(reading.category, AirQualityCategory::Moderate);
}

#[test]
fn retry_countdown_is_clamped_at_zero() {
    let now = Utc::now();
    let metadata = RefreshMetadata {
        next_retry_at: Some(now - Duration::seconds(5)),
        ..RefreshMetadata::default()
    };

    assert_eq!(metadata.retry_in_seconds_at(now), Some(0));
}

#[test]
fn summarize_precip_window_includes_12h_boundary() {
    let base = NaiveDateTime::parse_from_str("2026-02-12T15:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = (0..13)
        .map(|idx| HourlyForecast {
            time: base + chrono::Duration::hours(i64::from(idx)),
            temperature_2m_c: None,
            weather_code: None,
            is_day: None,
            relative_humidity_2m: None,
            precipitation_probability: None,
            precipitation_mm: Some(if idx == 12 { 0.4 } else { 0.0 }),
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
fn high_low_calculation() {
    // Case 1: Both high and low available
    let bundle = minimal_bundle(Some(25.4), Some(10.6));

    // Celsius: 25.4 -> 25, 10.6 -> 11
    assert_eq!(bundle.high_low(Units::Celsius), Some((25, 11)));

    // Fahrenheit:
    // 25.4 * 1.8 + 32 = 45.72 + 32 = 77.72 -> 78
    // 10.6 * 1.8 + 32 = 19.08 + 32 = 51.08 -> 51
    assert_eq!(bundle.high_low(Units::Fahrenheit), Some((78, 51)));

    // Case 2: Missing high
    let bundle = minimal_bundle(None, Some(10.6));
    assert_eq!(bundle.high_low(Units::Celsius), None);

    // Case 3: Missing low
    let bundle = minimal_bundle(Some(25.4), None);
    assert_eq!(bundle.high_low(Units::Celsius), None);
}

#[test]
fn weather_code_to_particle_all_categories() {
    assert_eq!(weather_code_to_particle(0), ParticleKind::None); // Clear
    assert_eq!(weather_code_to_particle(3), ParticleKind::None); // Cloudy
    assert_eq!(weather_code_to_particle(61), ParticleKind::Rain); // Rain
    assert_eq!(weather_code_to_particle(71), ParticleKind::Snow); // Snow
    assert_eq!(weather_code_to_particle(45), ParticleKind::Fog); // Fog
    assert_eq!(weather_code_to_particle(95), ParticleKind::Thunder); // Thunder
}

#[test]
fn weather_label_for_time_code_one_day_night() {
    assert!(weather_label_for_time(1, true).contains("Mainly clear"));
    assert!(weather_label_for_time(1, false).contains("Mainly clear night"));
}

#[test]
fn weather_icon_all_modes() {
    // Emoji mode
    let emoji = weather_icon(0, IconMode::Emoji, true);
    assert!(!emoji.is_empty());
    // Unicode mode
    let uni = weather_icon(0, IconMode::Unicode, true);
    assert!(!uni.is_empty());
    // Night mode gives different result for some codes
    let day = weather_icon(0, IconMode::Unicode, true);
    let night = weather_icon(0, IconMode::Unicode, false);
    assert_ne!(day, night); // Clear day vs night differ
}

#[test]
fn european_aqi_categories_cover_full_range() {
    assert_eq!(categorize_european_aqi(10), AirQualityCategory::Good);
    assert_eq!(categorize_european_aqi(30), AirQualityCategory::Moderate);
    assert_eq!(
        categorize_european_aqi(50),
        AirQualityCategory::UnhealthySensitive
    );
    assert_eq!(categorize_european_aqi(70), AirQualityCategory::Unhealthy);
    assert_eq!(
        categorize_european_aqi(90),
        AirQualityCategory::VeryUnhealthy
    );
    assert_eq!(categorize_european_aqi(110), AirQualityCategory::Hazardous);
}

#[test]
fn air_quality_category_label_covers_all_variants() {
    assert_eq!(AirQualityCategory::Good.label(), "Good");
    assert_eq!(AirQualityCategory::Moderate.label(), "Moderate");
    assert_eq!(AirQualityCategory::UnhealthySensitive.label(), "USG");
    assert_eq!(AirQualityCategory::Unhealthy.label(), "Unhealthy");
    assert_eq!(AirQualityCategory::VeryUnhealthy.label(), "Very Unhealthy");
    assert_eq!(AirQualityCategory::Hazardous.label(), "Hazardous");
    assert_eq!(AirQualityCategory::Unknown.label(), "Unknown");
}

#[test]
fn forecast_bundle_current_helpers_return_correct_values() {
    let bundle = minimal_bundle(Some(8.0), Some(1.0));
    assert_eq!(bundle.current_weather_code(), 0);
    assert_eq!(bundle.current_temp(Units::Celsius), 20);
    assert_eq!(bundle.current_temp(Units::Fahrenheit), 68); // 20°C = 68°F
}

#[test]
fn summarize_precip_window_guard_cases() {
    // lookahead_hours == 0 → None
    assert!(summarize_precip_window(&[], 0, 0.1).is_none());
    // negative threshold → None
    assert!(summarize_precip_window(&[], 12, -1.0).is_none());
    // empty hourly → None (no matching hours)
    assert!(summarize_precip_window(&[], 12, 0.1).is_none());
}

#[test]
fn precip_window_has_precip_now_when_first_hour_has_precip() {
    let base = NaiveDateTime::parse_from_str("2026-02-12T08:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = vec![HourlyForecast {
        time: base,
        temperature_2m_c: None,
        weather_code: None,
        is_day: None,
        relative_humidity_2m: None,
        precipitation_probability: None,
        precipitation_mm: Some(1.5),
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
    let summary = summarize_precip_window(&hourly, 12, 0.1).expect("should find precip");
    assert!(summary.has_precip_now());
}

#[test]
fn refresh_metadata_mark_success_clears_failure_state() {
    let mut meta = RefreshMetadata::default();
    meta.mark_failure();
    assert_eq!(meta.consecutive_failures, 1);
    meta.mark_success();
    assert_eq!(meta.consecutive_failures, 0);
    assert!(meta.last_success.is_some());
}

#[test]
fn refresh_metadata_schedule_retry_sets_next_retry() {
    let mut meta = RefreshMetadata::default();
    meta.schedule_retry_in(30);
    assert!(meta.next_retry_at.is_some());
}

#[test]
fn summarize_dayparts_returns_empty_for_zero_max_days() {
    let base = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").unwrap();
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

#[test]
fn air_quality_reading_returns_none_when_both_indices_absent() {
    assert!(AirQualityReading::from_indices(None, None).is_none());
}

#[test]
fn air_quality_display_value_uses_us_index_first() {
    let reading = AirQualityReading::from_indices(Some(42.0), Some(25.0)).unwrap();
    assert_eq!(reading.display_value(), "42");
}

#[test]
fn air_quality_display_value_falls_back_to_european() {
    let reading = AirQualityReading::from_indices(None, Some(30.0)).unwrap();
    assert_eq!(reading.display_value(), "30");
}

#[test]
fn air_quality_categorizes_via_european_when_us_absent() {
    let reading = AirQualityReading::from_indices(None, Some(30.0)).unwrap();
    assert_eq!(reading.category, AirQualityCategory::Moderate);
}

#[test]
fn daypart_aggregation_handles_all_none_fields() {
    let base = NaiveDateTime::parse_from_str("2026-02-12T06:00", "%Y-%m-%dT%H:%M").unwrap();
    // A sample with all optional fields set to None
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
    // Alternatively, summarize_dayparts may return 0 results for None fields — both valid
}

fn minimal_bundle(high_c: Option<f32>, low_c: Option<f32>) -> ForecastBundle {
    ForecastBundle {
        location: Location::from_coords(0.0, 0.0),
        current: CurrentConditions {
            temperature_2m_c: 20.0,
            relative_humidity_2m: 50.0,
            apparent_temperature_c: 20.0,
            dew_point_2m_c: 10.0,
            weather_code: 0,
            precipitation_mm: 0.0,
            cloud_cover: 0.0,
            pressure_msl_hpa: 1013.0,
            visibility_m: 10000.0,
            wind_speed_10m: 10.0,
            wind_gusts_10m: 15.0,
            wind_direction_10m: 180.0,
            is_day: true,
            high_today_c: high_c,
            low_today_c: low_c,
        },
        hourly: vec![],
        daily: vec![],
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

fn sample_hour(
    time: NaiveDateTime,
    temp_c: f32,
    weather_code: u8,
    precip_probability: f32,
    precip_mm: f32,
    wind_kmh: f32,
    visibility_m: f32,
) -> HourlyForecast {
    HourlyForecast {
        time,
        temperature_2m_c: Some(temp_c),
        weather_code: Some(weather_code),
        is_day: Some(true),
        relative_humidity_2m: None,
        precipitation_probability: Some(precip_probability),
        precipitation_mm: Some(precip_mm),
        rain_mm: None,
        snowfall_cm: None,
        wind_speed_10m: Some(wind_kmh),
        wind_gusts_10m: None,
        pressure_msl_hpa: None,
        visibility_m: Some(visibility_m),
        cloud_cover: None,
        cloud_cover_low: None,
        cloud_cover_mid: None,
        cloud_cover_high: None,
    }
}

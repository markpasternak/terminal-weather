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

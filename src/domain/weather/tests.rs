use super::*;

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

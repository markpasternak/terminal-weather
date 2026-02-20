use super::*;
use crate::domain::weather::{CurrentConditions, ForecastBundle, Location};
use chrono::{NaiveDate, Utc};

use super::layout::DailyLayout;
use super::summary::{WeekSummaryData, summarize_week};

#[test]
fn range_bounds_are_clamped() {
    let (start, end) = bar_bounds(-50.0, 80.0, -10.0, 40.0, 12);
    assert!(start <= 12);
    assert!(end <= 12);
    assert!(start <= end);
}

#[test]
fn daily_layout_changes_by_width() {
    let wide = DailyLayout::for_area(Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 10,
    });
    assert!(wide.show_icon);
    assert!(wide.show_bar);
    assert!(wide.show_header);

    let medium = DailyLayout::for_area(Rect {
        x: 0,
        y: 0,
        width: 44,
        height: 10,
    });
    assert!(!medium.show_icon);
    assert!(medium.show_bar);
    assert!(medium.show_header);

    let narrow = DailyLayout::for_area(Rect {
        x: 0,
        y: 0,
        width: 32,
        height: 10,
    });
    assert!(!narrow.show_icon);
    assert!(!narrow.show_bar);
    assert!(!narrow.show_header);
}

#[test]
fn summarize_week_aggregates_three_day_dataset() {
    let daily = sample_three_day_daily();
    let bundle = sample_bundle(daily.clone());
    let summary = summarize_week(&bundle, Units::Celsius);
    assert_three_day_summary(&summary, &daily);
}

fn sample_three_day_daily() -> Vec<DailyForecast> {
    vec![
        sample_day(DayInput {
            date: (2026, 2, 20),
            precip_mm: 3.0,
            rain_mm: 2.0,
            snow_cm: 0.0,
            gust_kmh: 30.0,
            uv: 4.5,
            min_c: -2.0,
            max_c: 6.0,
            daylight_s: 36000.0,
            sunshine_s: 18000.0,
            precip_hours: 2.0,
        }),
        sample_day(DayInput {
            date: (2026, 2, 21),
            precip_mm: 5.0,
            rain_mm: 1.5,
            snow_cm: 1.2,
            gust_kmh: 52.0,
            uv: 7.0,
            min_c: -4.0,
            max_c: 9.0,
            daylight_s: 43200.0,
            sunshine_s: 21600.0,
            precip_hours: 4.0,
        }),
        sample_day(DayInput {
            date: (2026, 2, 22),
            precip_mm: 2.0,
            rain_mm: 1.0,
            snow_cm: 0.0,
            gust_kmh: 22.0,
            uv: 5.2,
            min_c: 0.0,
            max_c: 4.0,
            daylight_s: 32400.0,
            sunshine_s: 10800.0,
            precip_hours: 1.0,
        }),
    ]
}

fn assert_three_day_summary(summary: &WeekSummaryData, daily: &[DailyForecast]) {
    assert!((summary.precip_total - 10.0).abs() < f32::EPSILON);
    assert!((summary.rain_total - 4.5).abs() < f32::EPSILON);
    assert!((summary.snow_total - 1.2).abs() < f32::EPSILON);
    assert_eq!(summary.avg_daylight, "10:20");
    assert_eq!(summary.avg_sun, "04:40");
    assert_eq!(summary.precip_hours_avg, "2.3h/day");
    assert_eq!(
        summary.wettest_txt,
        format!("{} 5.0mm", daily[1].date.format("%a"))
    );
    assert_eq!(
        summary.breeziest_txt,
        format!("{} 14m/s", daily[1].date.format("%a"))
    );
    assert_eq!(
        summary.uv_peak,
        format!("{} 7.0", daily[1].date.format("%a"))
    );
    assert_eq!(summary.week_thermal, "-4°..9°");
    assert_eq!(summary.highs, vec![6.0, 9.0, 4.0]);
    assert_eq!(summary.precip, vec![3.0, 5.0, 2.0]);
    assert_eq!(summary.gusts.len(), 3);
    assert!((summary.gusts[0] - 8.333_333).abs() < 0.001);
    assert!((summary.gusts[1] - 14.444_445).abs() < 0.001);
    assert!((summary.gusts[2] - 6.111_111).abs() < 0.001);
}

#[derive(Clone, Copy)]
struct DayInput {
    date: (i32, u32, u32),
    precip_mm: f32,
    rain_mm: f32,
    snow_cm: f32,
    gust_kmh: f32,
    uv: f32,
    min_c: f32,
    max_c: f32,
    daylight_s: f32,
    sunshine_s: f32,
    precip_hours: f32,
}

fn sample_day(input: DayInput) -> DailyForecast {
    DailyForecast {
        date: NaiveDate::from_ymd_opt(input.date.0, input.date.1, input.date.2)
            .expect("valid date"),
        weather_code: Some(3),
        temperature_max_c: Some(input.max_c),
        temperature_min_c: Some(input.min_c),
        sunrise: None,
        sunset: None,
        uv_index_max: Some(input.uv),
        precipitation_probability_max: Some(70.0),
        precipitation_sum_mm: Some(input.precip_mm),
        rain_sum_mm: Some(input.rain_mm),
        snowfall_sum_cm: Some(input.snow_cm),
        precipitation_hours: Some(input.precip_hours),
        wind_gusts_10m_max: Some(input.gust_kmh),
        daylight_duration_s: Some(input.daylight_s),
        sunshine_duration_s: Some(input.sunshine_s),
    }
}

fn sample_bundle(daily: Vec<DailyForecast>) -> ForecastBundle {
    ForecastBundle {
        location: Location::from_coords(59.3293, 18.0686),
        current: CurrentConditions {
            temperature_2m_c: 2.0,
            relative_humidity_2m: 75.0,
            apparent_temperature_c: 0.0,
            dew_point_2m_c: -1.0,
            weather_code: 3,
            precipitation_mm: 0.0,
            cloud_cover: 60.0,
            pressure_msl_hpa: 1010.0,
            visibility_m: 9000.0,
            wind_speed_10m: 12.0,
            wind_gusts_10m: 18.0,
            wind_direction_10m: 180.0,
            is_day: true,
            high_today_c: Some(6.0),
            low_today_c: Some(-2.0),
        },
        hourly: Vec::new(),
        daily,
        fetched_at: Utc::now(),
    }
}

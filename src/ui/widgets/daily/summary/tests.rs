use super::*;
use crate::cli::ThemeArg;
use crate::domain::weather::{CurrentConditions, Location, WeatherCategory};
use crate::ui::theme::{ColorCapability, theme_for};
use chrono::{NaiveDate, Utc};

fn sample_day(precip_mm: f32, gust_kmh: f32, uv: f32) -> DailyForecast {
    DailyForecast {
        date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("date"),
        weather_code: Some(3),
        temperature_max_c: Some(8.0),
        temperature_min_c: Some(1.0),
        sunrise: None,
        sunset: None,
        uv_index_max: Some(uv),
        precipitation_probability_max: Some(50.0),
        precipitation_sum_mm: Some(precip_mm),
        rain_sum_mm: Some(precip_mm),
        snowfall_sum_cm: Some(0.0),
        precipitation_hours: Some(1.0),
        wind_gusts_10m_max: Some(gust_kmh),
        daylight_duration_s: Some(36_000.0),
        sunshine_duration_s: Some(18_000.0),
    }
}

fn sample_bundle_with_daily(daily: Vec<DailyForecast>) -> ForecastBundle {
    ForecastBundle {
        location: Location::from_coords(59.33, 18.07),
        current: CurrentConditions {
            temperature_2m_c: 5.0,
            relative_humidity_2m: 60.0,
            apparent_temperature_c: 3.0,
            dew_point_2m_c: 1.0,
            weather_code: 3,
            precipitation_mm: 0.0,
            cloud_cover: 50.0,
            pressure_msl_hpa: 1010.0,
            visibility_m: 9000.0,
            wind_speed_10m: 8.0,
            wind_gusts_10m: 12.0,
            wind_direction_10m: 270.0,
            is_day: true,
            high_today_c: Some(8.0),
            low_today_c: Some(1.0),
        },
        hourly: vec![],
        daily,
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

fn test_theme() -> crate::ui::theme::Theme {
    theme_for(
        WeatherCategory::Clear,
        true,
        ColorCapability::Basic16,
        ThemeArg::Auto,
    )
}

// ── average_duration ─────────────────────────────────────────────────────

#[test]
fn average_duration_returns_placeholder_for_zero_count() {
    assert_eq!(average_duration(0.0, 0), "--:--");
}

#[test]
fn average_duration_computes_correctly() {
    // 7200s / 2 = 3600s = 1h00m
    assert_eq!(average_duration(7200.0, 2), "01:00");
}

// ── average_precip_hours ─────────────────────────────────────────────────

#[test]
fn average_precip_hours_returns_placeholder_for_zero_count() {
    assert_eq!(average_precip_hours(0.0, 0), "--");
}

#[test]
fn average_precip_hours_formats_correctly() {
    let result = average_precip_hours(6.0, 3);
    assert_eq!(result, "2.0h/day");
}

// ── format_day_value_mm / format_day_value_mps ───────────────────────────

#[test]
fn format_day_value_mm_none_returns_placeholder() {
    assert_eq!(format_day_value_mm(None), "--");
}

#[test]
fn format_day_value_mm_some_formats_correctly() {
    let result = format_day_value_mm(Some(("Mon".to_string(), 3.2)));
    assert!(result.contains("Mon"), "got: {result}");
    assert!(result.contains("3.2mm"), "got: {result}");
}

#[test]
fn format_day_value_mps_none_returns_placeholder() {
    assert_eq!(format_day_value_mps(None), "--");
}

#[test]
fn format_day_value_mps_some_formats_in_mps() {
    let result = format_day_value_mps(Some(("Tue".to_string(), 36.0)));
    assert!(result.contains("Tue"), "got: {result}");
    assert!(result.contains("m/s"), "got: {result}");
}

// ── format_uv_peak ───────────────────────────────────────────────────────

#[test]
fn format_uv_peak_none_returns_placeholder() {
    assert_eq!(format_uv_peak(None), "--");
}

// ── week_thermal_span ────────────────────────────────────────────────────

#[test]
fn week_thermal_span_missing_data_returns_placeholder() {
    assert_eq!(week_thermal_span(None, None, Units::Celsius), "--");
    assert_eq!(week_thermal_span(Some(1.0), None, Units::Celsius), "--");
}

#[test]
fn week_thermal_span_celsius_range() {
    let result = week_thermal_span(Some(-5.0), Some(15.0), Units::Celsius);
    assert!(result.contains("-5°"), "got: {result}");
    assert!(result.contains("15°"), "got: {result}");
}

// ── summarize_week ───────────────────────────────────────────────────────

#[test]
fn summarize_week_empty_daily_returns_defaults() {
    let bundle = sample_bundle_with_daily(vec![]);
    let summary = summarize_week(&bundle, Units::Celsius);
    assert_eq!(summary.avg_daylight, "--:--");
    assert_eq!(summary.avg_sun, "--:--");
    assert_eq!(summary.wettest_txt, "--");
    assert_eq!(summary.breeziest_txt, "--");
    assert_eq!(summary.uv_peak, "--");
    assert_eq!(summary.week_thermal, "--");
    assert!(summary.highs.is_empty());
}

#[test]
fn summarize_week_aggregates_single_day() {
    let daily = vec![sample_day(3.5, 20.0, 4.5)];
    let bundle = sample_bundle_with_daily(daily);
    let summary = summarize_week(&bundle, Units::Celsius);
    assert!((summary.precip_total - 3.5).abs() < f32::EPSILON);
    assert_eq!(summary.highs.len(), 1);
}

#[test]
fn summarize_week_null_precipitation_skipped() {
    let mut day = sample_day(0.0, 20.0, 3.0);
    day.precipitation_sum_mm = None;
    let bundle = sample_bundle_with_daily(vec![day]);
    let summary = summarize_week(&bundle, Units::Celsius);
    // wettest_txt should be "--" since no precip data
    assert_eq!(summary.wettest_txt, "--");
}

#[test]
fn append_week_profiles_covers_guard_and_render_paths() {
    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let summary = summarize_week(&bundle, Units::Celsius);
    let theme = test_theme();

    let mut lines = Vec::new();
    let mut rows = 4;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 70, 10),
        &summary,
        theme,
    );
    assert!(lines.is_empty());

    let mut lines = Vec::new();
    let mut rows = 3;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 80, 10),
        &summary,
        theme,
    );
    assert!(lines.is_empty());

    let mut lines = Vec::new();
    let mut rows = 4;
    append_week_profiles(
        &mut lines,
        &mut rows,
        Rect::new(0, 0, 80, 10),
        &summary,
        theme,
    );
    assert_eq!(lines.len(), 4);
    assert_eq!(rows, 0);
}

#[test]
fn append_profile_line_returns_when_no_rows_left() {
    let theme = test_theme();
    let values = vec![1.0_f32];
    let mut lines = Vec::new();
    let mut rows = 0;
    append_profile_line(
        &mut lines,
        &mut rows,
        ProfileLineSpec {
            label: "L",
            values: &values,
            suffix: "ok".to_string(),
            color: theme.text,
        },
        10,
        theme,
    );
    assert!(lines.is_empty());
    assert_eq!(rows, 0);
}

#[test]
fn append_profile_line_returns_when_values_empty() {
    let theme = test_theme();
    let empty: Vec<f32> = Vec::new();
    let mut lines = Vec::new();
    let mut rows = 1;
    append_profile_line(
        &mut lines,
        &mut rows,
        ProfileLineSpec {
            label: "L",
            values: &empty,
            suffix: "ok".to_string(),
            color: theme.text,
        },
        10,
        theme,
    );
    assert!(lines.is_empty());
    assert_eq!(rows, 1);
}

#[test]
fn append_profile_line_renders_and_consumes_row_when_values_present() {
    let theme = test_theme();
    let values = vec![1.0_f32];
    let mut lines = Vec::new();
    let mut rows = 1;
    append_profile_line(
        &mut lines,
        &mut rows,
        ProfileLineSpec {
            label: "L",
            values: &values,
            suffix: "ok".to_string(),
            color: theme.text,
        },
        10,
        theme,
    );
    assert_eq!(lines.len(), 1);
    assert_eq!(rows, 0);
}

#[test]
fn append_compact_profiles_covers_return_and_skip_paths() {
    let theme = test_theme();
    let summary = WeekSummaryData {
        precip_total: 2.0,
        breeziest_txt: "Mon 5m/s".to_string(),
        week_thermal: "1°..8°".to_string(),
        highs: vec![8.0],
        precip: vec![2.0],
        gusts: vec![5.0],
        ..WeekSummaryData::default()
    };

    let mut lines = Vec::new();
    append_compact_profiles(&mut lines, 0, Rect::new(0, 0, 40, 10), &summary, theme);
    assert!(lines.is_empty());

    append_compact_profiles(&mut lines, 2, Rect::new(0, 0, 20, 10), &summary, theme);
    assert!(lines.is_empty());

    append_compact_profiles(&mut lines, 1, Rect::new(0, 0, 40, 10), &summary, theme);
    assert_eq!(lines.len(), 1);

    let sparse = WeekSummaryData {
        gusts: vec![5.0],
        ..WeekSummaryData::default()
    };
    let mut sparse_lines = Vec::new();
    append_compact_profiles(
        &mut sparse_lines,
        2,
        Rect::new(0, 0, 40, 10),
        &sparse,
        theme,
    );
    assert_eq!(sparse_lines.len(), 1);
}

#[test]
fn render_week_summary_returns_early_on_narrow_width() {
    use ratatui::{Terminal, backend::TestBackend};

    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let theme = test_theme();
    let area = Rect::new(0, 0, 15, 10);

    let backend = TestBackend::new(15, 10);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| render_week_summary(frame, area, &bundle, Units::Celsius, theme))
        .expect("draw");
}

#[test]
fn render_week_summary_returns_early_on_zero_height() {
    use ratatui::{Terminal, backend::TestBackend};

    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let theme = test_theme();
    let area = Rect::new(0, 0, 80, 0);

    let backend = TestBackend::new(80, 0);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| render_week_summary(frame, area, &bundle, Units::Celsius, theme))
        .expect("draw");
}

#[test]
fn render_week_summary_returns_early_on_empty_daily() {
    use ratatui::{Terminal, backend::TestBackend};

    let bundle = sample_bundle_with_daily(vec![]);
    let theme = test_theme();
    let area = Rect::new(0, 0, 80, 10);

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| render_week_summary(frame, area, &bundle, Units::Celsius, theme))
        .expect("draw");
}

#[test]
fn week_thermal_span_fahrenheit() {
    let result = week_thermal_span(Some(-5.0), Some(15.0), Units::Fahrenheit);
    assert!(result.contains('°'));
}

#[test]
fn render_week_summary_hits_meta_and_sunrise_layout_paths() {
    use ratatui::{Terminal, backend::TestBackend};

    let bundle = sample_bundle_with_daily(vec![sample_day(1.0, 10.0, 2.0)]);
    let theme = test_theme();

    let mut wide_terminal = Terminal::new(TestBackend::new(80, 10)).expect("terminal");
    wide_terminal
        .draw(|frame| {
            render_week_summary(
                frame,
                Rect::new(0, 0, 80, 10),
                &bundle,
                Units::Celsius,
                theme,
            );
        })
        .expect("draw");

    let mut medium_terminal = Terminal::new(TestBackend::new(48, 10)).expect("terminal");
    medium_terminal
        .draw(|frame| {
            render_week_summary(
                frame,
                Rect::new(0, 0, 48, 10),
                &bundle,
                Units::Celsius,
                theme,
            );
        })
        .expect("draw");
}

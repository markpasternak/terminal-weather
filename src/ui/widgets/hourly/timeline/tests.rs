use super::*;
use crate::{
    cli::ThemeArg,
    domain::weather::WeatherCategory,
    ui::theme::{ColorCapability, theme_for},
};
use chrono::{NaiveDate, NaiveDateTime};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

#[test]
fn timeline_lines_use_single_time_axis_with_anchors() {
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let series = TimelineSeries {
        temps: vec![Some(-2.0), Some(4.0), Some(1.0)],
        temp_unit: "C",
        precips: vec![0.0, 0.8, 1.6],
        times: vec![dt(2026, 2, 22, 0), dt(2026, 2, 22, 6), dt(2026, 2, 22, 12)],
    };

    let lines = timeline_lines(&series, 56, 6, theme);
    assert_eq!(lines.len(), 3);

    let text = lines.iter().map(line_text).collect::<Vec<_>>();
    assert!(text[0].starts_with("Temp  "));
    assert!(text[1].starts_with("Precip"));
    assert!(text[2].starts_with("Time  "));
    assert!(text[0].contains('C'));
    assert!(text[0].contains(".."));
    assert!(text[1].contains("mm/h"));
    assert!(!text.iter().any(|line| line.starts_with("Tick  ")));
    assert!(!text.iter().any(|line| line.starts_with("Hour  ")));
    assert!(!text.iter().any(|line| line.starts_with("Shift ")));
}

#[test]
fn time_axis_labels_are_not_repeated_per_column() {
    let times = (0..13)
        .map(|hour| dt(2026, 2, 22, hour))
        .collect::<Vec<_>>();
    let axis = time_axis_line(&times, 80);

    assert_eq!(axis.matches("00").count(), 1);
    assert_eq!(axis.matches("06").count(), 1);
    assert_eq!(axis.matches("12").count(), 1);
}

#[test]
fn time_axis_marks_day_boundaries() {
    let times = (18..31)
        .map(|h| {
            let day = if h < 24 { 22 } else { 23 };
            let hour = h % 24;
            dt(2026, 2, day, hour)
        })
        .collect::<Vec<_>>();
    let axis = time_axis_line(&times, 64);
    assert!(axis.contains('|'));
}

#[test]
fn capped_precip_barline_uses_absolute_intensity_scale() {
    let bars = barline_capped(&[0.2, 1.0, 2.5], 3, 2.0);
    let chars = bars.chars().collect::<Vec<_>>();

    assert_eq!(chars.len(), 3);
    assert_ne!(chars[1], '█');
    assert_eq!(chars[2], '█');
}

#[test]
fn render_temp_precip_timeline_short_circuits_on_tiny_areas() {
    let theme = test_theme();
    let hours = [sample_hour(dt(2026, 2, 22, 6))];
    let slice: Vec<&HourlyForecast> = hours.iter().collect();
    let mut terminal = Terminal::new(TestBackend::new(20, 4)).expect("terminal");
    let mut stats = TimelineStats {
        wind_avg: Some(99.0),
        precip_prob_max: Some(99.0),
        cloud_avg: Some(99.0),
    };

    terminal
        .draw(|frame| {
            stats = render_temp_precip_timeline(
                frame,
                Rect::new(0, 0, 20, 0),
                &slice,
                theme,
                Units::Celsius,
            );
        })
        .expect("draw");
    assert_eq!(stats.wind_avg, None);

    terminal
        .draw(|frame| {
            stats = render_temp_precip_timeline(
                frame,
                Rect::new(0, 0, 11, 4),
                &slice,
                theme,
                Units::Celsius,
            );
        })
        .expect("draw");
    assert_eq!(stats.precip_prob_max, None);
    assert_eq!(stats.cloud_avg, None);
}

#[test]
fn render_chart_metrics_short_circuits_on_zero_dimensions() {
    let theme = test_theme();
    let stats = TimelineStats {
        wind_avg: Some(5.0),
        precip_prob_max: Some(60.0),
        cloud_avg: Some(40.0),
    };
    let mut terminal = Terminal::new(TestBackend::new(20, 4)).expect("terminal");

    terminal
        .draw(|frame| render_chart_metrics(frame, Rect::new(0, 0, 0, 1), stats, theme))
        .expect("draw");
    terminal
        .draw(|frame| render_chart_metrics(frame, Rect::new(0, 0, 20, 0), stats, theme))
        .expect("draw");
}

#[test]
fn timeline_lines_omit_time_axis_for_two_rows() {
    let theme = test_theme();
    let series = TimelineSeries {
        temps: vec![Some(1.0), Some(3.0)],
        temp_unit: "C",
        precips: vec![0.0, 0.5],
        times: vec![dt(2026, 2, 22, 3), dt(2026, 2, 22, 9)],
    };
    let lines = timeline_lines(&series, 40, 2, theme);

    assert_eq!(lines.len(), 2);
    assert!(line_text(&lines[0]).starts_with("Temp  "));
    assert!(line_text(&lines[1]).starts_with("Precip"));
}

#[test]
fn timeline_line_falls_back_to_range_when_chart_has_no_room() {
    let theme = test_theme();
    let series = TimelineSeries {
        temps: vec![Some(1.0), Some(4.0)],
        temp_unit: "C",
        precips: vec![0.1, 1.7],
        times: vec![dt(2026, 2, 22, 0), dt(2026, 2, 22, 6)],
    };
    let temp_line = line_text(&temp_timeline_line(&series, 4, theme));
    let rain_line = line_text(&rain_timeline_line(&series, 4, theme));

    assert!(temp_line.starts_with("Temp  "));
    assert!(!temp_line.contains("  1..4C"));
    assert!(rain_line.starts_with("Precip"));
    assert!(!rain_line.contains("mm/h"));
}

#[test]
fn average_and_sample_helpers_handle_small_inputs() {
    assert_eq!(average(std::iter::empty()), None);
    assert_eq!(average([1.0, 2.0, 3.0].into_iter()), Some(2.0));
    assert_eq!(sample_index(4, 1, 6), 0);
    assert_eq!(sample_index(4, 6, 1), 0);
    assert_eq!(sample_column(2, 1, 6), 0);
    assert_eq!(sample_column(2, 6, 1), 0);
}

#[test]
fn sparkline_and_barline_handle_empty_inputs() {
    assert_eq!(sparkline_optional(&[], 5), "");
    assert_eq!(sparkline_optional(&[Some(1.0)], 0), "");
    assert_eq!(sparkline_optional(&[None, None], 3), "···");

    assert_eq!(barline_capped(&[], 5, 2.0), "");
    assert_eq!(barline_capped(&[0.4], 0, 2.0), "");
}

#[test]
fn range_labels_and_time_axis_handle_empty_inputs() {
    assert_eq!(temp_range_label(&[], "C"), "--..--C");
    assert_eq!(precip_range_label(&[]), "--..--mm/h");
    assert_eq!(time_axis_line(&[], 10), "");
    assert_eq!(time_axis_line(&[dt(2026, 2, 22, 6)], 0), "");
}

#[test]
fn time_axis_handles_dense_sampling_and_day_change_labels() {
    let dense = (0..10)
        .map(|hour| dt(2026, 2, 22, hour))
        .collect::<Vec<_>>();
    let compact = time_axis_line(&dense, 2);
    assert_eq!(compact.chars().count(), 2);

    let crossing = vec![dt(2026, 2, 22, 21), dt(2026, 2, 23, 0), dt(2026, 2, 23, 3)];
    let axis = time_axis_line(&crossing, 12);
    assert!(axis.contains('|'));
    assert!(axis.contains("00"));
}

#[test]
fn time_axis_label_clips_at_right_edge() {
    assert_eq!(time_axis_line(&[dt(2026, 2, 22, 6)], 1), "0");
}

fn test_theme() -> Theme {
    theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    )
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
        temperature_2m_c: Some(3.0),
        weather_code: Some(3),
        is_day: Some(true),
        relative_humidity_2m: Some(70.0),
        precipitation_probability: Some(20.0),
        precipitation_mm: Some(0.2),
        rain_mm: Some(0.2),
        snowfall_cm: Some(0.0),
        wind_speed_10m: Some(9.0),
        wind_gusts_10m: Some(14.0),
        pressure_msl_hpa: Some(1008.0),
        visibility_m: Some(9000.0),
        cloud_cover: Some(55.0),
        cloud_cover_low: Some(20.0),
        cloud_cover_mid: Some(20.0),
        cloud_cover_high: Some(15.0),
    }
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

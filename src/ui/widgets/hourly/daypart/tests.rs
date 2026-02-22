use chrono::NaiveDate;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

use super::*;
use crate::{
    app::state::AppState,
    cli::ThemeArg,
    domain::weather::{Daypart, DaypartSummary, WeatherCategory},
    ui::theme::{ColorCapability, theme_for},
};

#[test]
fn daypart_visibility_correct_for_all_height_bands() {
    assert_eq!(daypart_visibility(0), (false, false, false));
    assert_eq!(daypart_visibility(3), (false, false, false));
    assert_eq!(daypart_visibility(4), (false, false, true));
    assert_eq!(daypart_visibility(5), (false, true, true));
    assert_eq!(daypart_visibility(6), (true, true, true));
    assert_eq!(daypart_visibility(99), (true, true, true));
}

#[test]
fn frozen_precip_code_detection_covers_snow_and_ice() {
    assert!(is_frozen_precip_code(71));
    assert!(is_frozen_precip_code(66));
    assert!(is_frozen_precip_code(85));
    assert!(!is_frozen_precip_code(61));
    assert!(!is_frozen_precip_code(3));
}

#[test]
fn daypart_advisory_covers_all_outcomes() {
    let mut summary = summary_for(Daypart::Morning, date(2026, 2, 22));

    summary.precip_sum_mm = 2.1;
    summary.weather_code = 61;
    assert_eq!(daypart_advisory(&summary), "Rain gear");

    summary.precip_sum_mm = 0.0;
    summary.precip_probability_max = Some(65.0);
    summary.weather_code = 71;
    assert_eq!(daypart_advisory(&summary), "Winter gear");

    summary.precip_probability_max = Some(10.0);
    summary.wind_max_kmh = Some(55.0);
    summary.visibility_median_m = Some(8_000.0);
    summary.temp_min_c = Some(5.0);
    assert_eq!(daypart_advisory(&summary), "Windy");

    summary.wind_max_kmh = Some(20.0);
    summary.visibility_median_m = Some(3_500.0);
    assert_eq!(daypart_advisory(&summary), "Low vis");

    summary.visibility_median_m = Some(9_000.0);
    summary.temp_min_c = Some(-2.0);
    assert_eq!(daypart_advisory(&summary), "Cold");

    summary.temp_min_c = Some(4.0);
    assert_eq!(daypart_advisory(&summary), "Steady");
}

#[test]
fn build_daypart_rows_respects_optional_sections() {
    let state = test_state();
    let theme = test_theme();
    let parts = full_day(date(2026, 2, 22));

    assert_eq!(
        build_daypart_rows(&parts, &state, theme, false, false, false).len(),
        3
    );
    assert_eq!(
        build_daypart_rows(&parts, &state, theme, true, false, false).len(),
        5
    );
    assert_eq!(
        build_daypart_rows(&parts, &state, theme, true, true, true).len(),
        7
    );
}

#[test]
fn temp_range_formats_all_option_shapes() {
    let mut summary = summary_for(Daypart::Noon, date(2026, 2, 22));
    summary.temp_min_c = Some(-1.0);
    summary.temp_max_c = Some(4.0);
    assert_eq!(temp_range(&summary, Units::Celsius), "-1-4°");

    summary.temp_min_c = Some(2.0);
    summary.temp_max_c = None;
    assert_eq!(temp_range(&summary, Units::Celsius), "2°");

    summary.temp_min_c = None;
    summary.temp_max_c = None;
    assert_eq!(temp_range(&summary, Units::Celsius), "--");
}

#[test]
fn truncate_handles_short_and_long_values() {
    assert_eq!(truncate("Breeze", 10), "Breeze");
    assert_eq!(truncate("Visibility improving", 6), "Visib…");
}

#[test]
fn can_render_daypart_cards_enforces_minimum_size() {
    assert!(!can_render_daypart_cards(Rect::new(0, 0, 23, 3)));
    assert!(!can_render_daypart_cards(Rect::new(0, 0, 24, 2)));
    assert!(can_render_daypart_cards(Rect::new(0, 0, 24, 3)));
}

#[test]
fn collect_parts_for_date_keeps_daypart_order() {
    let target = date(2026, 2, 22);
    let next_day = date(2026, 2, 23);
    let summaries = vec![
        summary_for(Daypart::Night, target),
        summary_for(Daypart::Morning, next_day),
        summary_for(Daypart::Morning, target),
        summary_for(Daypart::Evening, target),
    ];

    let parts = collect_parts_for_date(&summaries, target);
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].daypart, Daypart::Morning);
    assert_eq!(parts[1].daypart, Daypart::Evening);
    assert_eq!(parts[2].daypart, Daypart::Night);
}

#[test]
fn render_daypart_cards_returns_false_for_small_area() {
    let state = test_state();
    let theme = test_theme();
    let bundle = crate::test_support::sample_bundle();
    let mut terminal = Terminal::new(TestBackend::new(40, 12)).expect("terminal");
    let mut rendered = true;

    terminal
        .draw(|frame| {
            rendered =
                render_daypart_cards(frame, Rect::new(0, 0, 20, 2), &bundle, &state, theme, 1);
        })
        .expect("draw");

    assert!(!rendered);
}

#[test]
fn render_daypart_cards_returns_false_when_no_dates() {
    let state = test_state();
    let theme = test_theme();
    let bundle = crate::test_support::sample_bundle();
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).expect("terminal");
    let mut rendered = true;

    terminal
        .draw(|frame| {
            rendered =
                render_daypart_cards(frame, Rect::new(0, 0, 40, 8), &bundle, &state, theme, 0);
        })
        .expect("draw");

    assert!(!rendered);
}

#[test]
fn render_daypart_cards_returns_false_when_sections_too_short() {
    let state = test_state();
    let theme = test_theme();
    let mut bundle = crate::test_support::sample_bundle();
    bundle.hourly = vec![
        sample_hour_at(2026, 2, 22, 9),
        sample_hour_at(2026, 2, 23, 9),
    ];

    let mut terminal = Terminal::new(TestBackend::new(60, 20)).expect("terminal");
    let mut rendered = true;
    terminal
        .draw(|frame| {
            rendered =
                render_daypart_cards(frame, Rect::new(0, 0, 24, 3), &bundle, &state, theme, 2);
        })
        .expect("draw");

    assert!(!rendered);
}

#[test]
fn render_daypart_section_returns_true_when_some_parts_missing() {
    let state = test_state();
    let theme = test_theme();
    let target_date = date(2026, 2, 22);
    let summaries = vec![
        summary_for(Daypart::Morning, target_date),
        summary_for(Daypart::Evening, target_date),
    ];
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).expect("terminal");
    let mut rendered = false;

    terminal
        .draw(|frame| {
            rendered = render_daypart_section(
                frame,
                Rect::new(0, 0, 40, 6),
                target_date,
                &summaries,
                &state,
                theme,
            );
        })
        .expect("draw");

    assert!(rendered);
}

fn test_state() -> AppState {
    AppState::new(&crate::test_support::state_test_cli())
}

fn test_theme() -> Theme {
    theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    )
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

fn sample_hour_at(year: i32, month: u32, day: u32, hour: u32) -> HourlyForecast {
    let mut hour_sample = crate::test_support::sample_hourly();
    hour_sample.time = date(year, month, day)
        .and_hms_opt(hour, 0, 0)
        .expect("valid time");
    hour_sample
}

fn summary_for(daypart: Daypart, date: NaiveDate) -> DaypartSummary {
    DaypartSummary {
        date,
        daypart,
        weather_code: 3,
        temp_min_c: Some(1.0),
        temp_max_c: Some(6.0),
        wind_min_kmh: Some(10.0),
        wind_max_kmh: Some(20.0),
        precip_sum_mm: 0.4,
        precip_probability_max: Some(30.0),
        visibility_median_m: Some(10_000.0),
        sample_count: 1,
    }
}

fn full_day(date: NaiveDate) -> Vec<DaypartSummary> {
    Daypart::all()
        .iter()
        .copied()
        .map(|daypart| summary_for(daypart, date))
        .collect::<Vec<_>>()
}

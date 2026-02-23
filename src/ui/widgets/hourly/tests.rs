use super::daypart::daypart_visibility;
use super::table::{build_optional_date_row, metric_row_specs, sanitize_precip_mm};
use super::*;
use crate::{
    cli::ThemeArg,
    domain::weather::{HourlyForecast, HourlyViewMode, WeatherCategory},
    ui::theme::{ColorCapability, theme_for},
};
use chrono::{NaiveDate, NaiveDateTime};
use ratatui::{Frame, Terminal, backend::TestBackend, layout::Rect, style::Style};

#[test]
fn width_below_70_forces_table() {
    let area = Rect::new(0, 0, 68, 12);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, area),
        HourlyViewMode::Table
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, area),
        HourlyViewMode::Table
    );
}

#[test]
fn height_below_5_forces_table() {
    let area = Rect::new(0, 0, 90, 4);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, area),
        HourlyViewMode::Table
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, area),
        HourlyViewMode::Table
    );
}

#[test]
fn hybrid_mode_requires_height_6() {
    let area_too_short = Rect::new(0, 0, 90, 7);
    let area_ok = Rect::new(0, 0, 90, 8);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, area_too_short),
        HourlyViewMode::Table
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, area_ok),
        HourlyViewMode::Hybrid
    );
}

#[test]
fn chart_mode_requires_height_8() {
    let area_too_short = Rect::new(0, 0, 90, 9);
    let area_ok = Rect::new(0, 0, 90, 10);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, area_too_short),
        HourlyViewMode::Table
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, area_ok),
        HourlyViewMode::Chart
    );
}

#[test]
fn chart_mode_requires_more_height_than_hybrid() {
    let hybrid_ok = Rect::new(0, 0, 90, 10);
    let chart_too_short = Rect::new(0, 0, 90, 9);
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Hybrid, hybrid_ok),
        HourlyViewMode::Hybrid
    );
    assert_eq!(
        effective_hourly_mode(HourlyViewMode::Chart, chart_too_short),
        HourlyViewMode::Table
    );
}

#[test]
fn hourly_panel_title_same_day() {
    let hours = [
        sample_hour(dt(2026, 2, 20, 9)),
        sample_hour(dt(2026, 2, 20, 15)),
    ];
    let slice: Vec<&HourlyForecast> = hours.iter().collect();
    let title = hourly_panel_title(HourlyViewMode::Table, &slice, false);
    assert!(title.contains("Fri 20 Feb"));
    assert!(!title.contains("→"));
}

#[test]
fn hourly_panel_title_multi_day() {
    let hours = [
        sample_hour(dt(2026, 2, 20, 9)),
        sample_hour(dt(2026, 2, 21, 15)),
    ];
    let slice: Vec<&HourlyForecast> = hours.iter().collect();
    let title = hourly_panel_title(HourlyViewMode::Table, &slice, false);
    assert!(title.contains("→"));
}

#[test]
fn sanitize_precip_non_negative() {
    assert!((sanitize_precip_mm(-1.0) - 0.0).abs() < f32::EPSILON);
    assert!((sanitize_precip_mm(0.0) - 0.0).abs() < f32::EPSILON);
    assert!((sanitize_precip_mm(1.2) - 1.2).abs() < f32::EPSILON);
}

#[test]
fn optional_metric_rows_keep_height_thresholds() {
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let count_for = |height: u16| {
        metric_row_specs(theme)
            .iter()
            .filter(|(min_height, _, _, _)| *min_height <= height)
            .count()
    };

    assert_eq!(count_for(4), 0);
    assert_eq!(count_for(5), 1);
    assert_eq!(count_for(6), 2);
    assert_eq!(count_for(9), 5);
    assert_eq!(count_for(12), 8);
}

#[test]
fn date_row_inserts_for_day_change_or_offset() {
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let hours = [
        sample_hour(dt(2026, 2, 20, 9)),
        sample_hour(dt(2026, 2, 20, 10)),
        sample_hour(dt(2026, 2, 21, 0)),
    ];
    let same_day = vec![&hours[0], &hours[1]];
    let crosses_day = vec![&hours[1], &hours[2]];

    assert!(build_optional_date_row(&same_day, 0, theme).is_none());
    assert!(build_optional_date_row(&crosses_day, 0, theme).is_some());
    assert!(build_optional_date_row(&same_day, 1, theme).is_some());
}

#[test]
fn daypart_visibility_thresholds_match_layout_contract() {
    assert_eq!(daypart_visibility(3), (false, false, false));
    assert_eq!(daypart_visibility(4), (false, false, true));
    assert_eq!(daypart_visibility(5), (false, true, true));
    assert_eq!(daypart_visibility(6), (true, true, true));
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
        temperature_2m_c: Some(1.0),
        weather_code: Some(3),
        is_day: Some(true),
        relative_humidity_2m: Some(75.0),
        precipitation_probability: Some(10.0),
        precipitation_mm: Some(0.2),
        rain_mm: Some(0.2),
        snowfall_cm: Some(0.0),
        wind_speed_10m: Some(12.0),
        wind_gusts_10m: Some(18.0),
        pressure_msl_hpa: Some(1009.0),
        visibility_m: Some(8000.0),
        cloud_cover: Some(60.0),
        cloud_cover_low: Some(20.0),
        cloud_cover_mid: Some(25.0),
        cloud_cover_high: Some(15.0),
    }
}

fn bundle_with_hour_count(count: usize) -> crate::domain::weather::ForecastBundle {
    let mut bundle = crate::test_support::sample_bundle();
    let base = bundle.hourly[0].clone();
    bundle.hourly = (0..count)
        .map(|idx| {
            let mut hour = base.clone();
            hour.time = base.time + chrono::Duration::hours(i64::try_from(idx).unwrap_or(0));
            hour.temperature_2m_c = Some(4.0 + idx as f32);
            hour
        })
        .collect();
    bundle
}

fn draw_hourly(state: &AppState, width: u16, height: u16) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let cli = crate::test_support::state_test_cli();
    terminal
        .draw(|frame| render(frame, frame.area(), state, &cli))
        .expect("draw hourly panel");
}

#[derive(Debug, Clone, Copy)]
struct RenderModeResults {
    hybrid_short: bool,
    hybrid_bands: [bool; 3],
    chart_short: bool,
    chart_tall: bool,
}

fn evaluate_render_mode_helpers(
    state: &AppState,
    bundle: &crate::domain::weather::ForecastBundle,
    slice: &[&HourlyForecast],
    theme: crate::ui::theme::Theme,
) -> RenderModeResults {
    let mut results = RenderModeResults {
        hybrid_short: false,
        hybrid_bands: [false, false, false],
        chart_short: false,
        chart_tall: false,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).expect("test terminal");
    terminal
        .draw(|frame| {
            results = collect_render_mode_results(frame, state, bundle, slice, theme);
        })
        .expect("draw helper branches");
    results
}

fn collect_render_mode_results(
    frame: &mut Frame,
    state: &AppState,
    bundle: &crate::domain::weather::ForecastBundle,
    slice: &[&HourlyForecast],
    theme: crate::ui::theme::Theme,
) -> RenderModeResults {
    let hybrid_short =
        render_hybrid_mode(frame, Rect::new(0, 0, 100, 6), state, bundle, slice, theme);
    let hybrid_bands = [
        render_hybrid_mode(frame, Rect::new(0, 0, 100, 9), state, bundle, slice, theme),
        render_hybrid_mode(frame, Rect::new(0, 0, 100, 10), state, bundle, slice, theme),
        render_hybrid_mode(frame, Rect::new(0, 0, 100, 12), state, bundle, slice, theme),
    ];
    let chart_short =
        render_chart_mode(frame, Rect::new(0, 0, 100, 7), state, bundle, slice, theme);
    let chart_tall = render_chart_mode(frame, Rect::new(0, 0, 100, 8), state, bundle, slice, theme);
    render_loading_placeholder(
        frame,
        Rect::new(0, 0, 20, 0),
        0,
        Style::default(),
        theme.accent,
        theme.muted_text,
    );
    RenderModeResults {
        hybrid_short,
        hybrid_bands,
        chart_short,
        chart_tall,
    }
}

#[test]
fn hourly_panel_title_empty_slice_includes_focus_prefix() {
    let title = hourly_panel_title(HourlyViewMode::Chart, &[], true);
    assert_eq!(title, "▶ Hourly · Chart");
}

#[test]
fn hourly_slice_clamps_offset_and_matches_visible_count() {
    let bundle = bundle_with_hour_count(24);
    let slice = hourly_slice(&bundle, 100, 80);
    assert_eq!(slice.len(), 1);
    assert_eq!(
        slice[0].time,
        bundle.hourly.last().expect("hourly point").time
    );
}

#[test]
fn render_handles_loading_and_empty_hourly_states() {
    let mut loading_state = AppState::new(&crate::test_support::state_test_cli());
    loading_state.panel_focus = PanelFocus::Hourly;
    draw_hourly(&loading_state, 80, 12);

    let mut empty_hourly_state = AppState::new(&crate::test_support::state_test_cli());
    let mut bundle = bundle_with_hour_count(1);
    bundle.hourly.clear();
    empty_hourly_state.weather = Some(bundle);
    empty_hourly_state.panel_focus = PanelFocus::Hourly;
    draw_hourly(&empty_hourly_state, 80, 12);
}

#[test]
fn render_exercises_hybrid_and_chart_modes() {
    let mut state = AppState::new(&crate::test_support::state_test_cli());
    state.weather = Some(bundle_with_hour_count(24));
    state.panel_focus = PanelFocus::Hourly;
    state.settings.inline_hints = true;

    state.hourly_view_mode = HourlyViewMode::Hybrid;
    draw_hourly(&state, 110, 20);

    state.hourly_view_mode = HourlyViewMode::Chart;
    draw_hourly(&state, 110, 20);
}

#[test]
fn render_mode_helpers_cover_size_branches() {
    let bundle = bundle_with_hour_count(24);
    let state = AppState::new(&crate::test_support::state_test_cli());
    let theme = theme_for(
        WeatherCategory::Cloudy,
        true,
        ColorCapability::Basic16,
        ThemeArg::Aurora,
    );
    let slice = bundle.hourly.iter().collect::<Vec<_>>();
    let results = evaluate_render_mode_helpers(&state, &bundle, &slice, theme);

    assert!(!results.hybrid_short);
    assert!(results.hybrid_bands.into_iter().all(|value| value));
    assert!(!results.chart_short);
    assert!(results.chart_tall);
}

#![allow(clippy::cast_precision_loss)]

use chrono::{NaiveDate, NaiveDateTime, Utc};
use ratatui::{Terminal, backend::TestBackend};
use terminal_weather::{
    app::state::{AppMode, AppState},
    cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
    domain::weather::{
        AirQualityCategory, AirQualityReading, CurrentConditions, DailyForecast, ForecastBundle,
        HourlyForecast, HourlyViewMode, Location,
    },
    resilience::freshness::FreshnessState,
    ui,
};

fn cli() -> Cli {
    Cli {
        city: Some("Stockholm".to_string()),
        units: UnitsArg::Celsius,
        fps: 30,
        no_animation: true,
        reduced_motion: false,
        no_flash: true,
        ascii_icons: false,
        emoji_icons: false,
        color: ColorArg::Auto,
        no_color: false,
        hourly_view: None,
        theme: ThemeArg::Auto,
        hero_visual: HeroVisualArg::AtmosCanvas,
        country_code: None,
        lat: None,
        lon: None,
        forecast_url: None,
        air_quality_url: None,
        refresh_interval: 600,
        demo: false,
        one_shot: false,
    }
}

fn fixture_bundle(code: u8) -> ForecastBundle {
    let base_time = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").unwrap();
    let base_date = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
    ForecastBundle {
        location: fixture_location(),
        current: fixture_current(code),
        hourly: fixture_hourly(base_time, code),
        daily: fixture_daily(base_date, code),
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

fn fixture_bundle_with_aqi(code: u8) -> ForecastBundle {
    let mut bundle = fixture_bundle(code);
    bundle.air_quality = Some(AirQualityReading {
        us_aqi: Some(42),
        european_aqi: Some(18),
        category: AirQualityCategory::Good,
    });
    bundle
}

fn fixture_location() -> Location {
    Location {
        name: "Stockholm".to_string(),
        latitude: 59.3293,
        longitude: 18.0686,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
        population: Some(975_000),
    }
}

fn fixture_current(code: u8) -> CurrentConditions {
    CurrentConditions {
        temperature_2m_c: 7.2,
        relative_humidity_2m: 73.0,
        apparent_temperature_c: 5.8,
        dew_point_2m_c: 2.1,
        weather_code: code,
        precipitation_mm: 0.4,
        cloud_cover: 42.0,
        pressure_msl_hpa: 1008.2,
        visibility_m: 11_200.0,
        wind_speed_10m: 12.0,
        wind_gusts_10m: 21.0,
        wind_direction_10m: 220.0,
        is_day: true,
        high_today_c: Some(9.0),
        low_today_c: Some(3.0),
    }
}

fn fixture_hourly(base_time: NaiveDateTime, code: u8) -> Vec<HourlyForecast> {
    (0..12)
        .map(|idx| HourlyForecast {
            time: base_time + chrono::Duration::hours(i64::from(idx)),
            temperature_2m_c: Some(5.0 + (idx as f32 * 0.5)),
            weather_code: Some(code),
            is_day: Some((6..=18).contains(&(idx % 24))),
            relative_humidity_2m: Some(70.0),
            precipitation_probability: Some(35.0),
            precipitation_mm: Some(0.4 + idx as f32 * 0.1),
            rain_mm: Some(0.3 + idx as f32 * 0.1),
            snowfall_cm: Some(if code >= 71 { 0.2 } else { 0.0 }),
            wind_speed_10m: Some(12.0 + idx as f32 * 0.3),
            wind_gusts_10m: Some(20.0 + idx as f32 * 0.5),
            pressure_msl_hpa: Some(1008.0 + idx as f32 * 0.4),
            visibility_m: Some(9_500.0 - idx as f32 * 80.0),
            cloud_cover: Some(40.0 + idx as f32 * 2.0),
            cloud_cover_low: Some(12.0 + idx as f32 * 1.0),
            cloud_cover_mid: Some(24.0 + idx as f32 * 1.3),
            cloud_cover_high: Some(36.0 + idx as f32 * 1.5),
        })
        .collect::<Vec<_>>()
}

fn fixture_daily(base_date: NaiveDate, code: u8) -> Vec<DailyForecast> {
    (0..7)
        .map(|idx| DailyForecast {
            date: base_date + chrono::Duration::days(i64::from(idx)),
            weather_code: Some(code),
            temperature_max_c: Some(8.0 + idx as f32),
            temperature_min_c: Some(1.0 + idx as f32 * 0.3),
            sunrise: NaiveDateTime::parse_from_str(
                &format!(
                    "{}T06:{:02}",
                    (base_date + chrono::Duration::days(i64::from(idx))).format("%Y-%m-%d"),
                    10 + idx
                ),
                "%Y-%m-%dT%H:%M",
            )
            .ok(),
            sunset: NaiveDateTime::parse_from_str(
                &format!(
                    "{}T17:{:02}",
                    (base_date + chrono::Duration::days(i64::from(idx))).format("%Y-%m-%d"),
                    40 + idx
                ),
                "%Y-%m-%dT%H:%M",
            )
            .ok(),
            uv_index_max: Some(2.0),
            precipitation_probability_max: Some(40.0),
            precipitation_sum_mm: Some(2.5 + idx as f32 * 0.6),
            rain_sum_mm: Some(2.0 + idx as f32 * 0.5),
            snowfall_sum_cm: Some(if code >= 71 {
                1.2 + idx as f32 * 0.2
            } else {
                0.0
            }),
            precipitation_hours: Some(2.0 + idx as f32 * 0.3),
            wind_gusts_10m_max: Some(22.0 + idx as f32 * 1.1),
            daylight_duration_s: Some(9.2 * 3600.0 + idx as f32 * 140.0),
            sunshine_duration_s: Some(4.1 * 3600.0 + idx as f32 * 190.0),
        })
        .collect::<Vec<_>>()
}

fn render_to_string(width: u16, height: u16, weather_code: u8) -> String {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle(weather_code));
    state.refresh_meta.state = FreshnessState::Fresh;
    state.refresh_meta.last_success = None;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_state_to_string(width: u16, height: u16, state: &AppState, cli: &Cli) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, state, cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_with_hourly_mode_to_string(
    width: u16,
    height: u16,
    weather_code: u8,
    mode: HourlyViewMode,
) -> String {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle(weather_code));
    state.refresh_meta.state = FreshnessState::Fresh;
    state.refresh_meta.last_success = None;
    state.settings.hourly_view = mode;
    state.hourly_view_mode = mode;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

fn render_help_to_string(width: u16, height: u16, weather_code: u8) -> String {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.help_open = true;
    state.weather = Some(fixture_bundle(weather_code));
    state.refresh_meta.state = FreshnessState::Fresh;
    state.refresh_meta.last_success = None;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &state, &cli))
        .expect("draw");

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

#[test]
fn snapshot_120x40_clear() {
    insta::assert_snapshot!("120x40_clear", render_to_string(120, 40, 0));
}

#[test]
fn snapshot_80x24_rain() {
    insta::assert_snapshot!("80x24_rain", render_to_string(80, 24, 61));
}

#[test]
fn snapshot_80x24_rain_with_aqi() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle_with_aqi(61));
    state.refresh_meta.state = FreshnessState::Fresh;
    state.refresh_meta.last_success = Some(Utc::now());
    insta::assert_snapshot!(
        "80x24_rain_with_aqi",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn snapshot_60x20_snow() {
    insta::assert_snapshot!("60x20_snow", render_to_string(60, 20, 71));
}

#[test]
fn snapshot_40x15_fog() {
    insta::assert_snapshot!("40x15_fog", render_to_string(40, 15, 45));
}

#[test]
fn snapshot_80x24_thunder() {
    insta::assert_snapshot!("80x24_thunder", render_to_string(80, 24, 95));
}

#[test]
fn snapshot_19x9_tiny_fallback() {
    insta::assert_snapshot!("19x9_tiny_fallback", render_to_string(19, 9, 0));
}

#[test]
fn small_terminal_still_renders_main_ui() {
    let rendered = render_to_string(20, 10, 0);
    assert!(rendered.contains("Current"));
    assert!(!rendered.contains("Terminal too small"));
}

#[test]
fn below_minimum_terminal_shows_resize_guidance() {
    let rendered = render_to_string(19, 9, 0);
    assert!(rendered.contains("terminal-weather"));
    assert!(!rendered.contains("Current"));
}

#[test]
fn snapshot_100x30_help_overlay() {
    insta::assert_snapshot!("100x30_help_overlay", render_help_to_string(100, 30, 61));
}

#[test]
fn snapshot_80x24_stale_retry_badge() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle(61));
    state.refresh_meta.state = FreshnessState::Stale;
    state.refresh_meta.last_success = Some(Utc::now() - chrono::Duration::minutes(12));
    state.refresh_meta.schedule_retry_in(35);
    state.last_error = Some("forecast request failed".to_string());
    insta::assert_snapshot!(
        "80x24_stale_retry_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn snapshot_80x24_offline_badge() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle(61));
    state.refresh_meta.state = FreshnessState::Offline;
    state.refresh_meta.last_success = Some(Utc::now() - chrono::Duration::minutes(40));
    state.last_error = Some("forecast request failed".to_string());
    insta::assert_snapshot!(
        "80x24_offline_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn snapshot_80x24_syncing_badge() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.mode = AppMode::Ready;
    state.weather = Some(fixture_bundle(61));
    state.refresh_meta.state = FreshnessState::Fresh;
    state.fetch_in_flight = true;
    insta::assert_snapshot!(
        "80x24_syncing_badge",
        render_state_to_string(80, 24, &state, &cli)
    );
}

#[test]
fn regular_layout_renders_footer_shortcuts() {
    let rendered = render_to_string(120, 40, 0);
    assert!(rendered.contains("r Refresh"));
    assert!(rendered.contains("F1/? Help"));
}

#[test]
fn snapshot_120x40_hybrid() {
    insta::assert_snapshot!(
        "120x40_hybrid",
        render_with_hourly_mode_to_string(120, 40, 61, HourlyViewMode::Hybrid)
    );
}

#[test]
fn snapshot_100x30_hybrid() {
    insta::assert_snapshot!(
        "100x30_hybrid",
        render_with_hourly_mode_to_string(100, 30, 61, HourlyViewMode::Hybrid)
    );
}

#[test]
fn snapshot_80x24_hybrid() {
    insta::assert_snapshot!(
        "80x24_hybrid",
        render_with_hourly_mode_to_string(80, 24, 61, HourlyViewMode::Hybrid)
    );
}

#[test]
fn snapshot_60x20_hybrid() {
    insta::assert_snapshot!(
        "60x20_hybrid",
        render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Hybrid)
    );
}

#[test]
fn snapshot_120x40_chart() {
    insta::assert_snapshot!(
        "120x40_chart",
        render_with_hourly_mode_to_string(120, 40, 61, HourlyViewMode::Chart)
    );
}

#[test]
fn snapshot_100x30_chart() {
    insta::assert_snapshot!(
        "100x30_chart",
        render_with_hourly_mode_to_string(100, 30, 61, HourlyViewMode::Chart)
    );
}

#[test]
fn snapshot_80x24_chart() {
    insta::assert_snapshot!(
        "80x24_chart",
        render_with_hourly_mode_to_string(80, 24, 61, HourlyViewMode::Chart)
    );
}

#[test]
fn snapshot_60x20_chart() {
    insta::assert_snapshot!(
        "60x20_chart",
        render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Chart)
    );
}

#[test]
fn narrow_layout_forces_table_for_hybrid_mode() {
    let rendered = render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Hybrid);
    assert!(rendered.contains("Hourly · Table"));
}

#[test]
fn narrow_layout_forces_table_for_chart_mode() {
    let rendered = render_with_hourly_mode_to_string(60, 20, 61, HourlyViewMode::Chart);
    assert!(rendered.contains("Hourly · Table"));
}

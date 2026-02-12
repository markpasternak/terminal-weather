use atmos_tui::{
    app::state::{AppMode, AppState},
    cli::{Cli, SilhouetteSourceArg, ThemeArg, UnitsArg},
    domain::weather::{CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location},
    resilience::freshness::FreshnessState,
    ui,
};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use ratatui::{Terminal, backend::TestBackend};

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
        theme: ThemeArg::Auto,
        silhouette_source: SilhouetteSourceArg::Local,
        country_code: None,
        lat: None,
        lon: None,
        refresh_interval: 600,
    }
}

fn fixture_bundle(code: u8) -> ForecastBundle {
    let location = Location {
        name: "Stockholm".to_string(),
        latitude: 59.3293,
        longitude: 18.0686,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
        population: Some(975_000),
    };

    let current = CurrentConditions {
        temperature_2m_c: 7.2,
        relative_humidity_2m: 73.0,
        apparent_temperature_c: 5.8,
        weather_code: code,
        wind_speed_10m: 12.0,
        wind_direction_10m: 220.0,
        is_day: true,
        high_today_c: Some(9.0),
        low_today_c: Some(3.0),
    };

    let base_time = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = (0..12)
        .map(|idx| HourlyForecast {
            time: base_time + chrono::Duration::hours(i64::from(idx)),
            temperature_2m_c: Some(5.0 + (idx as f32 * 0.5)),
            weather_code: Some(code),
            relative_humidity_2m: Some(70.0),
            precipitation_probability: Some(35.0),
        })
        .collect::<Vec<_>>();

    let base_date = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
    let daily = (0..7)
        .map(|idx| DailyForecast {
            date: base_date + chrono::Duration::days(i64::from(idx)),
            weather_code: Some(code),
            temperature_max_c: Some(8.0 + idx as f32),
            temperature_min_c: Some(1.0 + idx as f32 * 0.3),
            sunrise: None,
            sunset: None,
            uv_index_max: Some(2.0),
            precipitation_probability_max: Some(40.0),
        })
        .collect::<Vec<_>>();

    ForecastBundle {
        location,
        current,
        hourly,
        daily,
        fetched_at: Utc::now(),
    }
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

#[test]
fn snapshot_120x40_clear() {
    insta::assert_snapshot!("120x40_clear", render_to_string(120, 40, 0));
}

#[test]
fn snapshot_80x24_rain() {
    insta::assert_snapshot!("80x24_rain", render_to_string(80, 24, 61));
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

use atmos_tui::{
    app::{events::AppEvent, state::AppState},
    cli::{Cli, SilhouetteSourceArg, ThemeArg, UnitsArg},
    domain::weather::{
        CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location, Units,
    },
};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

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

fn fixture_bundle() -> ForecastBundle {
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
        weather_code: 61,
        wind_speed_10m: 12.0,
        wind_direction_10m: 220.0,
        is_day: true,
        high_today_c: Some(9.0),
        low_today_c: Some(3.0),
    };

    let base_time = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M").unwrap();
    let hourly = (0..24)
        .map(|idx| HourlyForecast {
            time: base_time + chrono::Duration::hours(i64::from(idx)),
            temperature_2m_c: Some(5.0 + (idx as f32 * 0.5)),
            weather_code: Some(61),
            relative_humidity_2m: Some(70.0),
            precipitation_probability: Some(35.0),
        })
        .collect::<Vec<_>>();

    let base_date = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
    let daily = (0..7)
        .map(|idx| DailyForecast {
            date: base_date + chrono::Duration::days(i64::from(idx)),
            weather_code: Some(61),
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

#[tokio::test]
async fn flow_unit_toggle_changes_display_units() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.weather = Some(fixture_bundle());
    let (tx, _rx) = mpsc::channel(8);

    state
        .handle_event(
            AppEvent::Input(Event::Key(KeyEvent::new(
                KeyCode::Char('f'),
                KeyModifiers::NONE,
            ))),
            &tx,
            &cli,
        )
        .await
        .unwrap();
    assert_eq!(state.units, Units::Fahrenheit);

    state
        .handle_event(
            AppEvent::Input(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::NONE,
            ))),
            &tx,
            &cli,
        )
        .await
        .unwrap();
    assert_eq!(state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_hourly_scroll_clamps_bounds() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.weather = Some(fixture_bundle());
    let (tx, _rx) = mpsc::channel(8);

    for _ in 0..50 {
        state
            .handle_event(
                AppEvent::Input(Event::Key(KeyEvent::new(
                    KeyCode::Right,
                    KeyModifiers::NONE,
                ))),
                &tx,
                &cli,
            )
            .await
            .unwrap();
    }

    assert!(state.hourly_offset <= 18);

    for _ in 0..50 {
        state
            .handle_event(
                AppEvent::Input(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))),
                &tx,
                &cli,
            )
            .await
            .unwrap();
    }

    assert_eq!(state.hourly_offset, 0);
}

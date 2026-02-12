use chrono::{NaiveDate, NaiveDateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use terminal_weather::{
    app::{events::AppEvent, state::AppState},
    cli::{Cli, HeroVisualArg, ThemeArg, UnitsArg},
    domain::weather::{
        CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location, Units,
    },
    ui::layout::visible_hour_count,
};
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
        hero_visual: HeroVisualArg::AtmosCanvas,
        country_code: None,
        lat: None,
        lon: None,
        refresh_interval: 600,
        demo: false,
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
        dew_point_2m_c: 2.1,
        weather_code: 61,
        precipitation_mm: 0.5,
        cloud_cover: 48.0,
        pressure_msl_hpa: 1006.8,
        visibility_m: 10_400.0,
        wind_speed_10m: 12.0,
        wind_gusts_10m: 20.0,
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
            precipitation_mm: Some(0.4),
            rain_mm: Some(0.4),
            snowfall_cm: Some(0.0),
            wind_speed_10m: Some(11.0),
            wind_gusts_10m: Some(18.0),
            pressure_msl_hpa: Some(1007.0),
            visibility_m: Some(9_800.0),
            cloud_cover: Some(45.0),
            cloud_cover_low: Some(15.0),
            cloud_cover_mid: Some(25.0),
            cloud_cover_high: Some(35.0),
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
            precipitation_sum_mm: Some(2.2),
            rain_sum_mm: Some(2.0),
            snowfall_sum_cm: Some(0.0),
            precipitation_hours: Some(2.5),
            wind_gusts_10m_max: Some(24.0),
            daylight_duration_s: Some(9.0 * 3600.0),
            sunshine_duration_s: Some(4.0 * 3600.0),
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

    let expected_max = state
        .weather
        .as_ref()
        .map(|bundle| {
            bundle
                .hourly
                .len()
                .saturating_sub(visible_hour_count(state.viewport_width))
        })
        .unwrap_or(0);
    assert!(state.hourly_offset <= expected_max);

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

#[tokio::test]
async fn flow_ctrl_c_quits_without_toggling_units() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    state.weather = Some(fixture_bundle());
    let (tx, mut rx) = mpsc::channel(8);

    state
        .handle_event(
            AppEvent::Input(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            ))),
            &tx,
            &cli,
        )
        .await
        .unwrap();

    let event = rx.recv().await.expect("quit event");
    assert!(matches!(event, AppEvent::Quit));
    assert_eq!(state.units, Units::Celsius);
}

#[tokio::test]
async fn flow_city_picker_keeps_dialog_open_while_typing_l() {
    let cli = cli();
    let mut state = AppState::new(&cli);
    let (tx, _rx) = mpsc::channel(8);
    state.city_picker_open = true;

    state
        .handle_event(
            AppEvent::Input(Event::Key(KeyEvent::new(
                KeyCode::Char('l'),
                KeyModifiers::NONE,
            ))),
            &tx,
            &cli,
        )
        .await
        .unwrap();

    assert!(state.city_picker_open);
    assert_eq!(state.city_query, "l");
}

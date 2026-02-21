#![allow(clippy::cast_precision_loss)]
#![allow(dead_code)]

use chrono::{NaiveDate, NaiveDateTime, Utc};
use terminal_weather::{
    app::state::{AppMode, AppState},
    cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
    domain::weather::{
        AirQualityCategory, AirQualityReading, CurrentConditions, DailyForecast, ForecastBundle,
        HourlyForecast, Location,
    },
    resilience::freshness::FreshnessState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureProfile {
    Snapshot,
    Flow,
}

pub fn stockholm_cli() -> Cli {
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

pub fn stockholm_location() -> Location {
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

pub fn fixture_bundle(profile: FixtureProfile, weather_code: u8) -> ForecastBundle {
    let base_time = NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M")
        .expect("valid fixed time");
    let base_date = NaiveDate::from_ymd_opt(2026, 2, 12).expect("valid fixed date");

    ForecastBundle {
        location: stockholm_location(),
        current: fixture_current(profile, weather_code),
        hourly: fixture_hourly(profile, base_time, weather_code),
        daily: fixture_daily(profile, base_date, weather_code),
        air_quality: None,
        fetched_at: Utc::now(),
    }
}

pub fn fixture_bundle_with_aqi(profile: FixtureProfile, weather_code: u8) -> ForecastBundle {
    let mut bundle = fixture_bundle(profile, weather_code);
    bundle.air_quality = Some(AirQualityReading {
        us_aqi: Some(42),
        european_aqi: Some(18),
        category: AirQualityCategory::Good,
    });
    bundle
}

pub fn state_with_weather(cli: &Cli, bundle: ForecastBundle) -> AppState {
    let mut state = AppState::new(cli);
    state.weather = Some(bundle);
    state
}

pub fn ready_state_with_weather(cli: &Cli, bundle: ForecastBundle) -> AppState {
    let mut state = state_with_weather(cli, bundle);
    state.mode = AppMode::Ready;
    state.refresh_meta.state = FreshnessState::Fresh;
    state.refresh_meta.last_success = None;
    state
}

pub fn assert_stockholm_cli_shape(cli: &Cli) {
    assert_eq!(cli.city.as_deref(), Some("Stockholm"));
    assert_eq!(cli.refresh_interval, 600);
    assert_eq!(cli.theme, ThemeArg::Auto);
    assert_eq!(cli.hero_visual, HeroVisualArg::AtmosCanvas);
    assert!(!cli.demo);
    assert!(!cli.one_shot);
}

pub fn assert_fixture_bundle_shape(
    bundle: &ForecastBundle,
    expected_hourly: usize,
    expected_daily: usize,
    expected_weather_code: u8,
) {
    assert_eq!(bundle.hourly.len(), expected_hourly);
    assert_eq!(bundle.daily.len(), expected_daily);
    assert_eq!(
        bundle.location.timezone.as_deref(),
        Some("Europe/Stockholm")
    );
    assert_eq!(bundle.current.weather_code, expected_weather_code);
}

fn fixture_current(profile: FixtureProfile, weather_code: u8) -> CurrentConditions {
    match profile {
        FixtureProfile::Snapshot => CurrentConditions {
            temperature_2m_c: 7.2,
            relative_humidity_2m: 73.0,
            apparent_temperature_c: 5.8,
            dew_point_2m_c: 2.1,
            weather_code,
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
        },
        FixtureProfile::Flow => CurrentConditions {
            temperature_2m_c: 7.2,
            relative_humidity_2m: 73.0,
            apparent_temperature_c: 5.8,
            dew_point_2m_c: 2.1,
            weather_code,
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
        },
    }
}

fn fixture_hourly(
    profile: FixtureProfile,
    base_time: NaiveDateTime,
    weather_code: u8,
) -> Vec<HourlyForecast> {
    match profile {
        FixtureProfile::Snapshot => (0..12)
            .map(|idx| HourlyForecast {
                time: base_time + chrono::Duration::hours(i64::from(idx)),
                temperature_2m_c: Some(5.0 + (idx as f32 * 0.5)),
                weather_code: Some(weather_code),
                is_day: Some((6..=18).contains(&(idx % 24))),
                relative_humidity_2m: Some(70.0),
                precipitation_probability: Some(35.0),
                precipitation_mm: Some(0.4 + idx as f32 * 0.1),
                rain_mm: Some(0.3 + idx as f32 * 0.1),
                snowfall_cm: Some(if weather_code >= 71 { 0.2 } else { 0.0 }),
                wind_speed_10m: Some(12.0 + idx as f32 * 0.3),
                wind_gusts_10m: Some(20.0 + idx as f32 * 0.5),
                pressure_msl_hpa: Some(1008.0 + idx as f32 * 0.4),
                visibility_m: Some(9_500.0 - idx as f32 * 80.0),
                cloud_cover: Some(40.0 + idx as f32 * 2.0),
                cloud_cover_low: Some(12.0 + idx as f32 * 1.0),
                cloud_cover_mid: Some(24.0 + idx as f32 * 1.3),
                cloud_cover_high: Some(36.0 + idx as f32 * 1.5),
            })
            .collect::<Vec<_>>(),
        FixtureProfile::Flow => (0..24)
            .map(|idx| HourlyForecast {
                time: base_time + chrono::Duration::hours(i64::from(idx)),
                temperature_2m_c: Some(5.0 + (idx as f32 * 0.5)),
                weather_code: Some(weather_code),
                is_day: Some((6..=18).contains(&(idx % 24))),
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
            .collect::<Vec<_>>(),
    }
}

fn fixture_daily(
    profile: FixtureProfile,
    base_date: NaiveDate,
    weather_code: u8,
) -> Vec<DailyForecast> {
    match profile {
        FixtureProfile::Snapshot => (0..7)
            .map(|idx| snapshot_daily_forecast(base_date, idx, weather_code))
            .collect::<Vec<_>>(),
        FixtureProfile::Flow => (0..7)
            .map(|idx| flow_daily_forecast(base_date, idx, weather_code))
            .collect::<Vec<_>>(),
    }
}

fn snapshot_daily_forecast(base_date: NaiveDate, idx: u32, weather_code: u8) -> DailyForecast {
    let date = base_date + chrono::Duration::days(i64::from(idx));
    DailyForecast {
        date,
        weather_code: Some(weather_code),
        temperature_max_c: Some(8.0 + idx as f32),
        temperature_min_c: Some(1.0 + idx as f32 * 0.3),
        sunrise: fixture_time(date, 6, 10 + idx),
        sunset: fixture_time(date, 17, 40 + idx),
        uv_index_max: Some(2.0),
        precipitation_probability_max: Some(40.0),
        precipitation_sum_mm: Some(2.5 + idx as f32 * 0.6),
        rain_sum_mm: Some(2.0 + idx as f32 * 0.5),
        snowfall_sum_cm: Some(if weather_code >= 71 {
            1.2 + idx as f32 * 0.2
        } else {
            0.0
        }),
        precipitation_hours: Some(2.0 + idx as f32 * 0.3),
        wind_gusts_10m_max: Some(22.0 + idx as f32 * 1.1),
        daylight_duration_s: Some(9.2 * 3600.0 + idx as f32 * 140.0),
        sunshine_duration_s: Some(4.1 * 3600.0 + idx as f32 * 190.0),
    }
}

fn flow_daily_forecast(base_date: NaiveDate, idx: u32, weather_code: u8) -> DailyForecast {
    DailyForecast {
        date: base_date + chrono::Duration::days(i64::from(idx)),
        weather_code: Some(weather_code),
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
    }
}

fn fixture_time(date: NaiveDate, hour: u32, minute: u32) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(
        &format!("{}T{hour:02}:{minute:02}", date.format("%Y-%m-%d")),
        "%Y-%m-%dT%H:%M",
    )
    .ok()
}

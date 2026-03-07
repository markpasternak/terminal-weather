use chrono::Timelike;

use super::super::auto::{auto_theme_from_bundle, auto_theme_preview};
use super::super::{ColorCapability, Theme, theme_preview};
use super::as_rgb;
use crate::{app::state::SettingsSelection, cli::ThemeArg, test_support::state_test_cli};

#[test]
fn auto_theme_phases_are_distinct() {
    let sunrise = theme_for_auto_fixture(
        "2026-02-12T06:30",
        Some("2026-02-12T06:30"),
        Some("2026-02-12T18:00"),
    );
    let noon = theme_for_auto_fixture(
        "2026-02-12T13:00",
        Some("2026-02-12T06:30"),
        Some("2026-02-12T18:00"),
    );
    let golden = theme_for_auto_fixture(
        "2026-02-12T17:20",
        Some("2026-02-12T06:30"),
        Some("2026-02-12T18:00"),
    );
    let night = theme_for_auto_fixture(
        "2026-02-12T22:30",
        Some("2026-02-12T06:30"),
        Some("2026-02-12T18:00"),
    );

    assert_ne!(as_rgb(sunrise.top), as_rgb(noon.top));
    assert_ne!(as_rgb(noon.bottom), as_rgb(golden.bottom));
    assert_ne!(as_rgb(golden.accent), as_rgb(night.accent));
}

#[test]
fn auto_theme_weather_overlays_are_distinct() {
    let clear = theme_for_precip_fixture(0.0, 0.0, 10_000.0, 1, 1);
    let rain = theme_for_precip_fixture(3.0, 0.0, 10_000.0, 61, 61);
    let snow = theme_for_precip_fixture(0.0, 2.0, 10_000.0, 71, 71);
    let thunder = theme_for_precip_fixture(2.0, 0.0, 10_000.0, 95, 95);

    assert_ne!(as_rgb(clear.bottom), as_rgb(rain.bottom));
    assert_ne!(as_rgb(rain.top), as_rgb(snow.top));
    assert_ne!(as_rgb(thunder.top), as_rgb(rain.top));
}

#[test]
fn auto_theme_clearing_soon_warms_the_accent() {
    let steady_rain = theme_for_transition_fixture(61, 61, 3.0);
    let clearing = theme_for_transition_fixture(61, 1, 3.0);
    assert_ne!(as_rgb(steady_rain.accent), as_rgb(clearing.accent));
}

#[test]
fn auto_theme_fallback_without_sun_times_still_varies_by_clock() {
    let early = theme_for_auto_fixture("2026-02-12T05:00", None, None);
    let late = theme_for_auto_fixture("2026-02-12T20:00", None, None);
    assert_ne!(as_rgb(early.top), as_rgb(late.top));
}

#[test]
fn auto_preview_reflects_conditions_and_theme_preview_reflects_explicit_theme() {
    let rainy_bundle = bundle_for_time(BundleFixture {
        now: "2026-02-12T19:45",
        sunrise: Some("2026-02-12T06:30"),
        sunset: Some("2026-02-12T18:00"),
        current_code: 61,
        precip_mm: 2.5,
        snow_cm: 0.0,
        visibility_m: 4000.0,
        incoming_code: 61,
    });
    let preview = auto_theme_preview(&rainy_bundle).expect("auto preview");
    assert!(preview.contains("Auto:"));

    let mut state = crate::app::state::AppState::new(&state_test_cli());
    state.settings_selected = SettingsSelection::Theme;
    state.settings.theme = ThemeArg::TokyoNightStorm;
    assert!(theme_preview(&state).contains("Tokyo Night Storm"));
}

fn theme_for_auto_fixture(now: &str, sunrise: Option<&str>, sunset: Option<&str>) -> Theme {
    let bundle = bundle_for_time(BundleFixture {
        now,
        sunrise,
        sunset,
        current_code: 1,
        precip_mm: 0.0,
        snow_cm: 0.0,
        visibility_m: 10_000.0,
        incoming_code: 1,
    });
    auto_theme_from_bundle(&bundle, ColorCapability::TrueColor).expect("auto theme")
}

fn theme_for_precip_fixture(
    precip_mm: f32,
    snow_cm: f32,
    visibility_m: f32,
    current_code: u8,
    incoming_code: u8,
) -> Theme {
    let bundle = bundle_for_time(BundleFixture {
        now: "2026-02-12T19:45",
        sunrise: Some("2026-02-12T06:30"),
        sunset: Some("2026-02-12T18:00"),
        current_code,
        precip_mm,
        snow_cm,
        visibility_m,
        incoming_code,
    });
    auto_theme_from_bundle(&bundle, ColorCapability::TrueColor).expect("auto theme")
}

fn theme_for_transition_fixture(current_code: u8, incoming_code: u8, precip_mm: f32) -> Theme {
    let bundle = bundle_for_time(BundleFixture {
        now: "2026-02-12T18:45",
        sunrise: Some("2026-02-12T06:30"),
        sunset: Some("2026-02-12T18:00"),
        current_code,
        precip_mm,
        snow_cm: 0.0,
        visibility_m: 10_000.0,
        incoming_code,
    });
    auto_theme_from_bundle(&bundle, ColorCapability::TrueColor).expect("auto theme")
}

#[derive(Clone, Copy)]
struct BundleFixture<'a> {
    now: &'a str,
    sunrise: Option<&'a str>,
    sunset: Option<&'a str>,
    current_code: u8,
    precip_mm: f32,
    snow_cm: f32,
    visibility_m: f32,
    incoming_code: u8,
}

fn bundle_for_time(fixture: BundleFixture<'_>) -> crate::domain::weather::ForecastBundle {
    let mut bundle = crate::test_support::sample_bundle();
    let now_dt =
        chrono::NaiveDateTime::parse_from_str(fixture.now, "%Y-%m-%dT%H:%M").expect("valid now");
    apply_current_fixture(&mut bundle, fixture, now_dt);
    apply_daily_fixture(
        &mut bundle,
        now_dt.date(),
        fixture.sunrise.map(parse_dt),
        fixture.sunset.map(parse_dt),
    );
    bundle.hourly = (0..6)
        .map(|idx| hourly_fixture(fixture, now_dt, idx, bundle.current.cloud_cover))
        .collect();

    bundle
}

fn apply_current_fixture(
    bundle: &mut crate::domain::weather::ForecastBundle,
    fixture: BundleFixture<'_>,
    now_dt: chrono::NaiveDateTime,
) {
    bundle.current.weather_code = fixture.current_code;
    bundle.current.precipitation_mm = fixture.precip_mm;
    bundle.current.visibility_m = fixture.visibility_m;
    bundle.current.cloud_cover = current_cloud_cover(fixture.current_code);
    bundle.current.is_day = is_daytime(now_dt);
}

fn apply_daily_fixture(
    bundle: &mut crate::domain::weather::ForecastBundle,
    date: chrono::NaiveDate,
    sunrise: Option<chrono::NaiveDateTime>,
    sunset: Option<chrono::NaiveDateTime>,
) {
    if let Some(day) = bundle.daily.first_mut() {
        day.date = date;
        day.sunrise = sunrise;
        day.sunset = sunset;
    }
}

fn hourly_fixture(
    fixture: BundleFixture<'_>,
    now_dt: chrono::NaiveDateTime,
    idx: i32,
    current_cloud_cover: f32,
) -> crate::domain::weather::HourlyForecast {
    let time = now_dt + chrono::Duration::hours(i64::from(idx));
    let mut hour = crate::test_support::sample_hourly();
    hour.time = time;
    hour.weather_code = Some(hourly_code(fixture, idx));
    hour.is_day = Some(is_daytime(time));
    hour.precipitation_mm = Some(hourly_value(fixture.precip_mm, idx));
    hour.snowfall_cm = Some(hourly_value(fixture.snow_cm, idx));
    hour.visibility_m = Some(fixture.visibility_m);
    hour.cloud_cover = Some(if idx == 0 { current_cloud_cover } else { 65.0 });
    hour.wind_gusts_10m = Some(if fixture.incoming_code == 95 {
        70.0
    } else {
        18.0
    });
    hour
}

fn hourly_code(fixture: BundleFixture<'_>, idx: i32) -> u8 {
    if idx == 0 {
        fixture.current_code
    } else {
        fixture.incoming_code
    }
}

fn hourly_value(value: f32, idx: i32) -> f32 {
    if idx == 0 { value } else { value / 2.0 }
}

fn current_cloud_cover(current_code: u8) -> f32 {
    if current_code == 3 { 70.0 } else { 35.0 }
}

fn is_daytime(time: chrono::NaiveDateTime) -> bool {
    time.hour() >= 6 && time.hour() < 19
}

fn parse_dt(value: &str) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").expect("valid time")
}

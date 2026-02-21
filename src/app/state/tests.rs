use super::{
    AppState, SettingsSelection, adjust_setting_selection, initial_selected_location, is_city_char,
};
use crate::{
    app::settings::{MotionSetting, RecentLocation, RuntimeSettings},
    cli::{Cli, ColorArg, HeroVisualArg, IconMode, ThemeArg, UnitsArg},
};
use std::fmt::Debug;
use std::sync::atomic::Ordering;

#[test]
fn city_input_accepts_unicode_letters() {
    assert!(is_city_char('å'));
    assert!(is_city_char('Å'));
    assert!(is_city_char('é'));
}

#[test]
fn city_input_rejects_control_chars() {
    assert!(!is_city_char('\n'));
    assert!(!is_city_char('\t'));
}

#[test]
fn initial_selected_location_uses_recent_when_no_cli_location() {
    let cli = test_cli();
    let mut settings = RuntimeSettings::default();
    settings.recent_locations.push(RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.3293,
        longitude: 18.0686,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    });

    let selected = initial_selected_location(&cli, &settings).expect("selected location");
    assert_eq!(selected.name, "Stockholm");
}

#[test]
fn initial_selected_location_respects_cli_city_override() {
    let mut cli = test_cli();
    cli.city = Some("Berlin".to_string());
    let mut settings = RuntimeSettings::default();
    settings.recent_locations.push(RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.3293,
        longitude: 18.0686,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    });

    assert!(initial_selected_location(&cli, &settings).is_none());
}

#[test]
fn settings_hint_explains_hero_visual() {
    let mut state = AppState::new(&test_cli());
    state.settings_selected = super::SettingsSelection::HeroVisual;
    assert!(state.settings_hint().contains("Current panel right side"));
}

#[test]
fn apply_runtime_settings_updates_refresh_interval_runtime() {
    let mut state = AppState::new(&test_cli());
    state.settings.refresh_interval_secs = 300;
    state.apply_runtime_settings();
    assert_eq!(
        state.refresh_interval_secs_runtime.load(Ordering::Relaxed),
        300
    );
}

#[test]
fn adjust_setting_cycles_forward_and_backward() {
    let mut state = AppState::new(&test_cli());
    assert_cycle(
        &mut state,
        SettingsSelection::Motion,
        MotionSetting::Off,
        MotionSetting::Full,
        MotionSetting::Reduced,
        MotionSetting::Full,
        |s| s.settings.motion,
    );
    assert_cycle(
        &mut state,
        SettingsSelection::Icons,
        IconMode::Unicode,
        IconMode::Ascii,
        IconMode::Emoji,
        IconMode::Ascii,
        |s| s.settings.icon_mode,
    );
    assert_cycle(
        &mut state,
        SettingsSelection::HeroVisual,
        HeroVisualArg::AtmosCanvas,
        HeroVisualArg::GaugeCluster,
        HeroVisualArg::SkyObservatory,
        HeroVisualArg::GaugeCluster,
        |s| s.settings.hero_visual,
    );
}

fn assert_cycle<T, F>(
    state: &mut AppState,
    selection: SettingsSelection,
    initial: T,
    next: T,
    next_after_wrap: T,
    previous: T,
    read: F,
) where
    T: Copy + Debug + PartialEq,
    F: Fn(&AppState) -> T,
{
    assert_eq!(read(state), initial);
    assert!(adjust_setting_selection(state, selection, 1));
    assert_eq!(read(state), next);
    assert!(adjust_setting_selection(state, selection, 1));
    assert_eq!(read(state), next_after_wrap);
    assert!(adjust_setting_selection(state, selection, -1));
    assert_eq!(read(state), previous);
}

fn test_cli() -> Cli {
    Cli {
        city: None,
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

use crate::cli::{HeroVisualArg, HourlyViewArg, IconMode, ThemeArg, UnitsArg};
use crate::domain::weather::HourlyViewMode;
use tempfile::NamedTempFile;

use super::{
    MotionSetting, RecentLocation, RuntimeSettings, clear_runtime_settings, hourly_view_from_cli,
    load_runtime_settings, save_runtime_settings,
};

#[test]
fn from_cli_defaults_basic_mapping() {
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.units = UnitsArg::Fahrenheit;
    cli.theme = ThemeArg::Nord;
    cli.no_flash = true;
    cli.hero_visual = HeroVisualArg::GaugeCluster;
    cli.refresh_interval = 300;

    let settings = RuntimeSettings::from_cli_defaults(&cli);

    assert_eq!(settings.units, crate::domain::weather::Units::Fahrenheit);
    assert_eq!(settings.theme, ThemeArg::Nord);
    assert!(settings.no_flash);
    assert_eq!(settings.hero_visual, HeroVisualArg::GaugeCluster);
    assert_eq!(settings.refresh_interval_secs, 300);

    // Also check defaults
    let cli_default = crate::test_support::settings_default_test_cli();
    let settings_default = RuntimeSettings::from_cli_defaults(&cli_default);
    assert_eq!(
        settings_default.units,
        crate::domain::weather::Units::Celsius
    );
}

#[test]
fn from_cli_defaults_motion_logic() {
    // Default -> Full
    let cli = crate::test_support::settings_default_test_cli();
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).motion,
        MotionSetting::Full
    );

    // reduced_motion -> Reduced
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.reduced_motion = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).motion,
        MotionSetting::Reduced
    );

    // no_animation -> Off
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.no_animation = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).motion,
        MotionSetting::Off
    );

    // no_animation takes precedence over reduced_motion
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.no_animation = true;
    cli.reduced_motion = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).motion,
        MotionSetting::Off
    );
}

#[test]
fn from_cli_defaults_icon_mode_logic() {
    // Default -> Unicode
    let cli = crate::test_support::settings_default_test_cli();
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).icon_mode,
        IconMode::Unicode
    );

    // emoji_icons -> Emoji
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.emoji_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).icon_mode,
        IconMode::Emoji
    );

    // ascii_icons -> Ascii
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.ascii_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).icon_mode,
        IconMode::Ascii
    );

    // ascii_icons takes precedence over emoji_icons
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.ascii_icons = true;
    cli.emoji_icons = true;
    assert_eq!(
        RuntimeSettings::from_cli_defaults(&cli).icon_mode,
        IconMode::Ascii
    );
}

#[test]
fn from_cli_defaults_hardcoded_fields() {
    let mut cli = crate::test_support::settings_default_test_cli();
    // Even if we set hourly_view in CLI, the implementation currently hardcodes it to Table.
    cli.hourly_view = Some(HourlyViewArg::Chart);

    let settings = RuntimeSettings::from_cli_defaults(&cli);

    assert_eq!(settings.hourly_view, HourlyViewMode::Table);
    assert!(settings.recent_locations.is_empty());
}

#[test]
fn same_place_handles_unicode_case() {
    let a = RecentLocation {
        name: "Åre".to_string(),
        latitude: 63.4,
        longitude: 13.1,
        country: Some("Sweden".to_string()),
        admin1: Some("Jämtland".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    let b = RecentLocation {
        name: "åre".to_string(),
        latitude: 63.41,
        longitude: 13.11,
        country: Some("sweden".to_string()),
        admin1: Some("Jämtland".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    assert!(a.same_place(&b));
}

#[test]
fn runtime_settings_roundtrip_preserves_hourly_view() {
    let settings = RuntimeSettings {
        hourly_view: HourlyViewMode::Chart,
        ..RuntimeSettings::default()
    };

    let file = NamedTempFile::new().expect("create temp settings file");
    let path = file.path();
    save_runtime_settings(path, &settings).expect("save settings");
    let content = std::fs::read_to_string(path).expect("read settings");
    let restored: RuntimeSettings = serde_json::from_str(&content).expect("parse settings");

    assert_eq!(restored.hourly_view, HourlyViewMode::Chart);
}

#[test]
fn hourly_view_from_cli_maps_all_variants() {
    assert_eq!(
        hourly_view_from_cli(HourlyViewArg::Table),
        HourlyViewMode::Table
    );
    assert_eq!(
        hourly_view_from_cli(HourlyViewArg::Hybrid),
        HourlyViewMode::Hybrid
    );
    assert_eq!(
        hourly_view_from_cli(HourlyViewArg::Chart),
        HourlyViewMode::Chart
    );
}

#[test]
fn clear_runtime_settings_is_ok_for_existing_and_missing_file() {
    let file = NamedTempFile::new().expect("create temp settings file");
    let path = file.path().to_path_buf();
    clear_runtime_settings(&path).expect("clears existing file");
    clear_runtime_settings(&path).expect("missing file still ok");
}

#[test]
fn load_runtime_settings_without_disk_returns_cli_defaults() {
    let cli = crate::test_support::settings_default_test_cli();
    let (settings, path) = load_runtime_settings(&cli, false);
    assert!(path.is_none());
    assert_eq!(settings.units, crate::domain::weather::Units::Celsius);
    assert_eq!(settings.theme, ThemeArg::Auto);
}

#[test]
fn recent_location_display_name_variants() {
    let full = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.3,
        longitude: 18.0,
        country: Some("Sweden".to_string()),
        admin1: Some("Stockholm".to_string()),
        timezone: Some("Europe/Stockholm".to_string()),
    };
    assert_eq!(full.display_name(), "Stockholm, Stockholm, Sweden");

    let country_only = RecentLocation {
        admin1: None,
        ..full.clone()
    };
    assert_eq!(country_only.display_name(), "Stockholm, Sweden");

    let name_only = RecentLocation {
        country: None,
        ..country_only
    };
    assert_eq!(name_only.display_name(), "Stockholm");
}

#[test]
fn from_location_and_to_location_roundtrip_core_fields() {
    let location = crate::test_support::stockholm_location();
    let recent = RecentLocation::from_location(&location);
    let restored = recent.to_location();
    assert_eq!(restored.name, location.name);
    assert!((restored.latitude - location.latitude).abs() < f64::EPSILON);
    assert!((restored.longitude - location.longitude).abs() < f64::EPSILON);
    assert_eq!(restored.timezone, location.timezone);
}

#[test]
fn override_theme_non_auto_updates_settings() {
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.theme = ThemeArg::Nord;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.theme, ThemeArg::Nord);
}

#[test]
fn override_icon_mode_emoji_sets_emoji() {
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.emoji_icons = true;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.icon_mode, IconMode::Emoji);
}

#[test]
fn override_refresh_interval_non_default_updates() {
    let mut cli = crate::test_support::settings_default_test_cli();
    cli.refresh_interval = 300;
    let settings = RuntimeSettings::from_cli_defaults(&cli);
    assert_eq!(settings.refresh_interval_secs, 300);
}

#[test]
fn display_name_both_none_returns_name_only() {
    let loc = RecentLocation {
        name: "Tokyo".to_string(),
        latitude: 35.68,
        longitude: 139.69,
        country: None,
        admin1: None,
        timezone: None,
    };
    assert_eq!(loc.display_name(), "Tokyo");
}

#[test]
fn same_place_lat_not_close_returns_false() {
    let a = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    let b = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.50,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_lon_not_close_returns_false() {
    let a = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    let b = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.20,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_name_mismatch_returns_false() {
    let a = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    let b = RecentLocation {
        name: "Gothenburg".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    assert!(!a.same_place(&b));
}

#[test]
fn same_place_country_mismatch_returns_false() {
    let a = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    };
    let b = RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Norway".to_string()),
        admin1: None,
        timezone: None,
    };
    assert!(!a.same_place(&b));
}

#[test]
fn load_runtime_settings_with_env_uses_custom_path() {
    let cli = crate::test_support::settings_default_test_cli();

    unsafe {
        std::env::set_var(
            "TERMINAL_WEATHER_CONFIG_DIR",
            "/tmp/test-terminal-weather-config",
        );
    }
    let (settings, path) = load_runtime_settings(&cli, true);
    unsafe {
        std::env::remove_var("TERMINAL_WEATHER_CONFIG_DIR");
    }

    assert!(path.is_some());
    assert!(path.unwrap().ends_with("settings.json"));
    assert_eq!(settings.units, crate::domain::weather::Units::Celsius);
}

#[test]
fn load_runtime_settings_with_atmos_env_uses_custom_path() {
    let cli = crate::test_support::settings_default_test_cli();

    unsafe {
        std::env::set_var("ATMOS_TUI_CONFIG_DIR", "/tmp/test-atmos-config");
    }
    let (settings, path) = load_runtime_settings(&cli, true);
    unsafe {
        std::env::remove_var("ATMOS_TUI_CONFIG_DIR");
    }

    assert!(path.is_some());
    assert!(path.unwrap().ends_with("settings.json"));
    assert_eq!(settings.units, crate::domain::weather::Units::Celsius);
}

#[test]
fn load_runtime_settings_with_disk_reads_saved() {
    let settings = RuntimeSettings {
        units: crate::domain::weather::Units::Fahrenheit,
        hourly_view: HourlyViewMode::Chart,
        ..RuntimeSettings::default()
    };

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let path = temp_dir.path().join("settings.json");
    save_runtime_settings(&path, &settings).expect("save settings");

    let cli = crate::test_support::settings_default_test_cli();

    unsafe {
        std::env::set_var("TERMINAL_WEATHER_CONFIG_DIR", temp_dir.path());
    }
    let (loaded, loaded_path) = load_runtime_settings(&cli, true);
    unsafe {
        std::env::remove_var("TERMINAL_WEATHER_CONFIG_DIR");
    }

    assert!(loaded_path.is_some());
    assert_eq!(loaded.units, crate::domain::weather::Units::Fahrenheit);
    assert_eq!(loaded.hourly_view, HourlyViewMode::Chart);
}

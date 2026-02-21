use super::*;

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
    let (settings, path) = load_runtime_settings(&default_cli(), false);
    assert!(path.is_none());
    assert_eq!(settings.units, crate::domain::weather::Units::Celsius);
    assert_eq!(settings.theme, ThemeArg::Auto);
}

#[test]
fn load_runtime_settings_with_override_uses_custom_path() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let (settings, path) = with_test_config_dir(temp_dir.path(), || {
        load_runtime_settings(&default_cli(), true)
    });

    assert_settings_json_path(path);
    assert_eq!(settings.units, crate::domain::weather::Units::Celsius);
}

#[test]
fn load_runtime_settings_with_explicit_override_path_reads_saved() {
    let settings = RuntimeSettings {
        units: crate::domain::weather::Units::Fahrenheit,
        hourly_view: HourlyViewMode::Chart,
        ..RuntimeSettings::default()
    };

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let path = temp_dir.path().join("settings.json");
    save_runtime_settings(&path, &settings).expect("save settings");

    let (loaded, loaded_path) = with_test_config_dir(temp_dir.path(), || {
        load_runtime_settings(&default_cli(), true)
    });

    assert!(loaded_path.is_some());
    assert_eq!(loaded.units, crate::domain::weather::Units::Fahrenheit);
    assert_eq!(loaded.hourly_view, HourlyViewMode::Chart);
}

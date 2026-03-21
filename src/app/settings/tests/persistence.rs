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

    // File exists - test creation and deletion
    assert!(path.exists(), "Tempfile should exist");
    clear_runtime_settings(&path).expect("clears existing file");
    assert!(!path.exists(), "Tempfile should be deleted");

    // File missing - should return Ok(()) without error
    clear_runtime_settings(&path).expect("missing file still ok");
}

#[test]
fn clear_runtime_settings_returns_error_on_failure() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    // Attempting to delete a directory as a file should result in an error
    let result = clear_runtime_settings(temp_dir.path());
    assert!(
        result.is_err(),
        "Expected an error when clearing a directory"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("clearing settings file failed"),
        "Error message did not contain expected context: {err_msg}"
    );
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

#[test]
fn deserialize_settings_without_update_fields_defaults_to_none() {
    let mut raw = serde_json::to_value(RuntimeSettings::default()).expect("serialize defaults");
    let object = raw.as_object_mut().expect("settings object");
    object.remove("last_update_check_unix");
    object.remove("last_seen_latest_version");
    let restored: RuntimeSettings =
        serde_json::from_value(raw).expect("parse legacy settings payload");
    assert!(restored.last_update_check_unix.is_none());
    assert!(restored.last_seen_latest_version.is_none());
}

#[test]
fn save_runtime_settings_succeeds_with_valid_path() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let valid_path = temp_dir.path().join("settings.json");

    let result = save_runtime_settings(&valid_path, &RuntimeSettings::default());

    assert!(result.is_ok());
    assert!(valid_path.exists());
}

#[test]
fn save_runtime_settings_fails_when_parent_is_file() {
    let file = NamedTempFile::new().expect("create temp file");
    let path_is_file = file.path().join("settings.json");

    let result = save_runtime_settings(&path_is_file, &RuntimeSettings::default());

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("creating settings directory failed"));
}

#[test]
fn save_runtime_settings_fails_when_path_is_directory() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let path_is_dir = temp_dir.path();

    let result = save_runtime_settings(path_is_dir, &RuntimeSettings::default());

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("opening settings file with strict permissions failed")
            || err_msg.contains("writing settings file failed")
            || err_msg.contains("Is a directory")
            || err_msg.contains("Access is denied")
    );
}

#[test]
fn load_runtime_settings_with_missing_file_uses_defaults() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let path = temp_dir.path().join("settings.json");
    assert!(!path.exists());

    let (loaded, loaded_path) = with_test_config_dir(temp_dir.path(), || {
        load_runtime_settings(&default_cli(), true)
    });

    assert!(loaded_path.is_some());
    assert_eq!(loaded.units, crate::domain::weather::Units::Celsius);
    assert_eq!(loaded.hourly_view, HourlyViewMode::Table);
}

#[test]
fn load_runtime_settings_with_invalid_json_uses_defaults() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let path = temp_dir.path().join("settings.json");
    std::fs::write(&path, "invalid json content").expect("write invalid json");

    let (loaded, loaded_path) = with_test_config_dir(temp_dir.path(), || {
        load_runtime_settings(&default_cli(), true)
    });

    assert!(loaded_path.is_some());
    assert_eq!(loaded.units, crate::domain::weather::Units::Celsius);
    assert_eq!(loaded.hourly_view, HourlyViewMode::Table);
}

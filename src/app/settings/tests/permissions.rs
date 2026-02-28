use super::*;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;

#[test]
#[cfg(unix)]
fn save_runtime_settings_sets_strict_permissions() {
    let settings = RuntimeSettings::default();
    let file = NamedTempFile::new().expect("create temp settings file");
    let path = file.path();

    // Remove file so save_runtime_settings creates it from scratch
    std::fs::remove_file(path).ok();

    save_runtime_settings(path, &settings).expect("save settings");

    let metadata = std::fs::metadata(path).expect("get metadata");
    let permissions = metadata.permissions();
    let mode = permissions.mode() & 0o777;

    assert_eq!(mode, 0o600, "Settings file should have 0600 permissions");
}

#[test]
#[cfg(unix)]
fn save_runtime_settings_creates_directory_with_strict_permissions() {
    let settings = RuntimeSettings::default();

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let config_dir = temp_dir.path().join("terminal-weather-test-dir");
    let path = config_dir.join("settings.json");

    assert!(!config_dir.exists(), "Test directory should not exist yet");

    save_runtime_settings(&path, &settings).expect("save settings");

    assert!(config_dir.exists(), "Directory should be created");

    let metadata = std::fs::metadata(&config_dir).expect("get metadata");
    let permissions = metadata.permissions();
    let mode = permissions.mode() & 0o777;

    assert_eq!(
        mode, 0o700,
        "Settings directory should have 0700 permissions"
    );
}

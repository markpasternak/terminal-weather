use std::path::{Path, PathBuf};

use crate::cli::{HeroVisualArg, HourlyViewArg, IconMode, ThemeArg, UnitsArg};
use crate::domain::weather::HourlyViewMode;
use tempfile::NamedTempFile;

use super::{
    MotionSetting, RecentLocation, RuntimeSettings, clear_runtime_settings, hourly_view_from_cli,
    load_runtime_settings, save_runtime_settings, with_settings_path_override,
};

fn default_cli() -> crate::cli::Cli {
    crate::test_support::settings_default_test_cli()
}

fn stockholm_recent_location() -> RecentLocation {
    RecentLocation {
        name: "Stockholm".to_string(),
        latitude: 59.33,
        longitude: 18.07,
        country: Some("Sweden".to_string()),
        admin1: None,
        timezone: None,
    }
}

fn named_recent_location(name: &str) -> RecentLocation {
    RecentLocation {
        name: name.to_string(),
        ..stockholm_recent_location()
    }
}

fn assert_settings_json_path(path: Option<PathBuf>) {
    let path = path.expect("settings path should be present");
    assert!(path.ends_with("settings.json"));
}

fn with_test_config_dir<R>(base: impl AsRef<Path>, run: impl FnOnce() -> R) -> R {
    let settings_path = base.as_ref().join("settings.json");
    with_settings_path_override(Some(settings_path), run)
}

mod cli_mapping;
mod persistence;
mod recent_location;

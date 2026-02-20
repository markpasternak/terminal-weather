use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{Cli, HeroVisualArg, HourlyViewArg, IconMode, ThemeArg, UnitsArg},
    domain::weather::{HourlyViewMode, Location, Units},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionSetting {
    Full,
    Reduced,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeSettings {
    pub units: Units,
    pub theme: ThemeArg,
    pub motion: MotionSetting,
    pub no_flash: bool,
    pub icon_mode: IconMode,
    pub hourly_view: HourlyViewMode,
    #[serde(default, alias = "silhouette_source", alias = "silhouetteSource")]
    pub hero_visual: HeroVisualArg,
    pub refresh_interval_secs: u64,
    pub recent_locations: Vec<RecentLocation>,
}

impl RuntimeSettings {
    #[must_use]
    pub fn from_cli_defaults(cli: &Cli) -> Self {
        let units = match cli.units {
            UnitsArg::Celsius => Units::Celsius,
            UnitsArg::Fahrenheit => Units::Fahrenheit,
        };
        let motion = if cli.no_animation {
            MotionSetting::Off
        } else if cli.reduced_motion {
            MotionSetting::Reduced
        } else {
            MotionSetting::Full
        };
        let icon_mode = if cli.ascii_icons {
            IconMode::Ascii
        } else if cli.emoji_icons {
            IconMode::Emoji
        } else {
            IconMode::Unicode
        };

        Self {
            units,
            theme: cli.theme,
            motion,
            no_flash: cli.no_flash,
            icon_mode,
            hourly_view: HourlyViewMode::Table,
            hero_visual: cli.hero_visual,
            refresh_interval_secs: cli.refresh_interval,
            recent_locations: Vec::new(),
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            units: Units::Celsius,
            theme: ThemeArg::Auto,
            motion: MotionSetting::Full,
            no_flash: false,
            icon_mode: IconMode::Unicode,
            hourly_view: HourlyViewMode::Table,
            hero_visual: HeroVisualArg::AtmosCanvas,
            refresh_interval_secs: 600,
            recent_locations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub timezone: Option<String>,
}

impl RecentLocation {
    #[must_use]
    pub fn from_location(location: &Location) -> Self {
        Self {
            name: location.name.clone(),
            latitude: location.latitude,
            longitude: location.longitude,
            country: location.country.clone(),
            admin1: location.admin1.clone(),
            timezone: location.timezone.clone(),
        }
    }

    #[must_use]
    pub fn to_location(&self) -> Location {
        Location {
            name: self.name.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            country: self.country.clone(),
            admin1: self.admin1.clone(),
            timezone: self.timezone.clone(),
            population: None,
        }
    }

    #[must_use]
    pub fn display_name(&self) -> String {
        match (&self.admin1, &self.country) {
            (Some(admin), Some(country)) => format!("{}, {}, {}", self.name, admin, country),
            (None, Some(country)) => format!("{}, {}", self.name, country),
            _ => self.name.clone(),
        }
    }

    #[must_use]
    pub fn same_place(&self, other: &Self) -> bool {
        let same_name = unicode_case_eq(&self.name, &other.name);
        let same_country = self
            .country
            .as_deref()
            .unwrap_or("")
            .eq_ignore_ascii_case(other.country.as_deref().unwrap_or(""));
        let close_lat = (self.latitude - other.latitude).abs() < 0.05;
        let close_lon = (self.longitude - other.longitude).abs() < 0.05;
        same_name && same_country && close_lat && close_lon
    }
}

fn unicode_case_eq(a: &str, b: &str) -> bool {
    fold_lower(a) == fold_lower(b)
}

fn fold_lower(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

#[must_use]
pub fn load_runtime_settings(cli: &Cli, enable_disk: bool) -> (RuntimeSettings, Option<PathBuf>) {
    let mut settings = RuntimeSettings::from_cli_defaults(cli);
    if !enable_disk {
        return (settings, None);
    }

    let Some(path) = settings_path() else {
        return (settings, None);
    };

    if let Ok(content) = fs::read_to_string(&path)
        && let Ok(saved) = serde_json::from_str::<RuntimeSettings>(&content)
    {
        settings = saved;
    }

    if cli.units != UnitsArg::Celsius {
        settings.units = Units::Fahrenheit;
    }
    if cli.theme != ThemeArg::Auto {
        settings.theme = cli.theme;
    }
    if cli.no_animation {
        settings.motion = MotionSetting::Off;
    } else if cli.reduced_motion {
        settings.motion = MotionSetting::Reduced;
    }
    if cli.no_flash {
        settings.no_flash = true;
    }
    if cli.ascii_icons {
        settings.icon_mode = IconMode::Ascii;
    } else if cli.emoji_icons {
        settings.icon_mode = IconMode::Emoji;
    }
    if cli.hero_visual != HeroVisualArg::AtmosCanvas {
        settings.hero_visual = cli.hero_visual;
    }
    if cli.refresh_interval != 600 {
        settings.refresh_interval_secs = cli.refresh_interval;
    }

    (settings, Some(path))
}

#[must_use]
pub fn hourly_view_from_cli(arg: HourlyViewArg) -> HourlyViewMode {
    match arg {
        HourlyViewArg::Table => HourlyViewMode::Table,
        HourlyViewArg::Hybrid => HourlyViewMode::Hybrid,
        HourlyViewArg::Chart => HourlyViewMode::Chart,
    }
}

pub fn save_runtime_settings(path: &Path, settings: &RuntimeSettings) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("creating settings directory failed")?;
    }
    let payload =
        serde_json::to_string_pretty(&settings).context("serializing settings payload failed")?;
    fs::write(path, payload).context("writing settings file failed")
}

pub fn clear_runtime_settings(path: &Path) -> anyhow::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).context("clearing settings file failed"),
    }
}

fn settings_path() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("TERMINAL_WEATHER_CONFIG_DIR") {
        return Some(PathBuf::from(base).join("settings.json"));
    }
    if let Some(base) = std::env::var_os("ATMOS_TUI_CONFIG_DIR") {
        return Some(PathBuf::from(base).join("settings.json"));
    }

    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("terminal-weather")
            .join("settings.json"),
    )
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::domain::weather::HourlyViewMode;

    use super::{RecentLocation, RuntimeSettings, save_runtime_settings};

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

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("tw-settings-{unique}.json"));
        save_runtime_settings(&path, &settings).expect("save settings");
        let content = std::fs::read_to_string(&path).expect("read settings");
        let restored: RuntimeSettings = serde_json::from_str(&content).expect("parse settings");
        let _ = std::fs::remove_file(&path);

        assert_eq!(restored.hourly_view, HourlyViewMode::Chart);
    }
}

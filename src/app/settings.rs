use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{Cli, HeroVisualArg, IconMode, ThemeArg, UnitsArg},
    domain::weather::{Location, Units},
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
    #[serde(default, alias = "silhouette_source", alias = "silhouetteSource")]
    pub hero_visual: HeroVisualArg,
    pub refresh_interval_secs: u64,
    pub recent_locations: Vec<RecentLocation>,
}

impl RuntimeSettings {
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

    pub fn display_name(&self) -> String {
        match (&self.admin1, &self.country) {
            (Some(admin), Some(country)) => format!("{}, {}, {}", self.name, admin, country),
            (None, Some(country)) => format!("{}, {}", self.name, country),
            _ => self.name.clone(),
        }
    }

    pub fn same_place(&self, other: &Self) -> bool {
        let same_name = self.name.eq_ignore_ascii_case(&other.name);
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

pub fn save_runtime_settings(path: &Path, settings: &RuntimeSettings) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("creating settings directory failed")?;
    }
    let payload =
        serde_json::to_string_pretty(&settings).context("serializing settings payload failed")?;
    fs::write(path, payload).context("writing settings file failed")
}

fn settings_path() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("ATMOS_TUI_CONFIG_DIR") {
        return Some(PathBuf::from(base).join("settings.json"));
    }

    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("atmos-tui")
            .join("settings.json"),
    )
}

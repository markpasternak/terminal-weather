use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{Cli, IconMode, SilhouetteSourceArg, ThemeArg, UnitsArg},
    domain::weather::Units,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionSetting {
    Full,
    Reduced,
    Off,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RuntimeSettings {
    pub units: Units,
    pub theme: ThemeArg,
    pub motion: MotionSetting,
    pub no_flash: bool,
    pub icon_mode: IconMode,
    pub silhouette_source: SilhouetteSourceArg,
    pub refresh_interval_secs: u64,
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
            silhouette_source: cli.silhouette_source,
            refresh_interval_secs: cli.refresh_interval,
        }
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
    if cli.silhouette_source != SilhouetteSourceArg::Auto {
        settings.silhouette_source = cli.silhouette_source;
    }
    if cli.refresh_interval != 600 {
        settings.refresh_interval_secs = cli.refresh_interval;
    }

    (settings, Some(path))
}

pub fn save_runtime_settings(path: &Path, settings: RuntimeSettings) -> anyhow::Result<()> {
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

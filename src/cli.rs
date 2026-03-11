#![allow(clippy::missing_errors_doc)]

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::ui::animation::MotionMode;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum UnitsArg {
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ColorArg {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum HourlyViewArg {
    Table,
    Hybrid,
    Chart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconMode {
    Unicode,
    Ascii,
    Emoji,
    NerdFont,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeArg {
    Auto,
    Aurora,
    MidnightCyan,
    #[serde(alias = "SlackAubergine", alias = "slack-aubergine")]
    Aubergine,
    #[serde(alias = "SlackHoth", alias = "slack-hoth")]
    Hoth,
    #[serde(alias = "SlackMonument", alias = "slack-monument")]
    Monument,
    Nord,
    CatppuccinMocha,
    Mono,
    HighContrast,
    Dracula,
    GruvboxMaterialDark,
    KanagawaWave,
    AyuMirage,
    AyuLight,
    PoimandresStorm,
    SelenizedDark,
    NoClownFiesta,
    TokyoNightStorm,
    RosePineMoon,
    EverforestDark,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HeroVisualArg {
    #[serde(alias = "Auto", alias = "auto")]
    #[default]
    AtmosCanvas,
    #[serde(alias = "Local", alias = "local")]
    GaugeCluster,
    #[serde(alias = "Web", alias = "web")]
    SkyObservatory,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser, Clone)]
#[command(
    name = "terminal-weather",
    version,
    about = "Animated terminal weather dashboard"
)]
pub struct Cli {
    /// City name. Interactive mode auto-detects via IP if omitted; --one-shot falls back to Stockholm.
    pub city: Option<String>,

    /// Default units
    #[arg(long, value_enum, default_value_t = UnitsArg::Celsius)]
    pub units: UnitsArg,

    /// Target FPS (15..60)
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u8).range(15..=60))]
    pub fps: u8,

    /// Disable particle animation
    #[arg(long)]
    pub no_animation: bool,

    /// Lower motion mode
    #[arg(long)]
    pub reduced_motion: bool,

    /// Motion system preset
    #[arg(long, value_enum)]
    pub motion: Option<MotionMode>,

    /// Disable thunder flash
    #[arg(long)]
    pub no_flash: bool,

    /// Force ASCII icons
    #[arg(long)]
    pub ascii_icons: bool,

    /// Force emoji icons
    #[arg(long)]
    pub emoji_icons: bool,

    /// Use Nerd Font weather icons
    #[arg(long)]
    pub nerd_font: bool,

    /// Color output policy
    #[arg(long, value_enum, default_value_t = ColorArg::Auto, conflicts_with = "no_color")]
    pub color: ColorArg,

    /// Alias for --color never
    #[arg(long, conflicts_with = "color")]
    pub no_color: bool,

    /// Hourly panel rendering mode
    #[arg(long, value_enum)]
    pub hourly_view: Option<HourlyViewArg>,

    /// Theme override
    #[arg(long, value_enum, default_value_t = ThemeArg::Auto)]
    pub theme: ThemeArg,

    /// Hero visual mode
    #[arg(long, value_enum, default_value_t = HeroVisualArg::AtmosCanvas)]
    pub hero_visual: HeroVisualArg,

    /// Geocode bias (ISO2)
    #[arg(long)]
    pub country_code: Option<String>,

    /// Direct latitude (requires --lon)
    #[arg(long)]
    pub lat: Option<f64>,

    /// Direct longitude (requires --lat)
    #[arg(long)]
    pub lon: Option<f64>,

    /// Override forecast API base URL
    #[arg(long)]
    pub forecast_url: Option<String>,

    /// Override air-quality API base URL
    #[arg(long)]
    pub air_quality_url: Option<String>,

    /// Refresh interval in seconds
    #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u64).range(10..=86400))]
    pub refresh_interval: u64,

    /// Run automated demo script and exit
    #[arg(long)]
    pub demo: bool,

    /// Print weather snapshot to stdout and exit (non-interactive)
    #[arg(long)]
    pub one_shot: bool,
}

impl Cli {
    #[must_use]
    pub fn default_city(&self) -> String {
        self.city.clone().unwrap_or_else(|| "Stockholm".to_string())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        match (self.lat, self.lon) {
            (Some(_), None) | (None, Some(_)) => {
                anyhow::bail!("--lat and --lon must be provided together")
            }
            _ => {}
        }

        if self.lat.is_some_and(|lat| !(-90.0..=90.0).contains(&lat)) {
            anyhow::bail!("Latitude must be between -90 and 90");
        }
        if self.lon.is_some_and(|lon| !(-180.0..=180.0).contains(&lon)) {
            anyhow::bail!("Longitude must be between -180 and 180");
        }

        Ok(())
    }

    #[must_use]
    pub fn effective_color_mode(&self) -> ColorArg {
        if self.no_color {
            ColorArg::Never
        } else {
            self.color
        }
    }

    #[must_use]
    pub fn effective_motion_mode(&self) -> MotionMode {
        self.motion
            .unwrap_or_else(|| MotionMode::from_legacy(self.no_animation, self.reduced_motion))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, ColorArg, HourlyViewArg};
    use crate::ui::animation::MotionMode;

    #[test]
    fn parses_color_enum_values() {
        let cli = Cli::parse_from(["terminal-weather", "--color", "always"]);
        assert_eq!(cli.color, ColorArg::Always);
        assert!(!cli.no_color);
    }

    #[test]
    fn parses_no_color_alias() {
        let cli = Cli::parse_from(["terminal-weather", "--no-color"]);
        assert!(cli.no_color);
        assert_eq!(cli.effective_color_mode(), ColorArg::Never);
    }

    #[test]
    fn rejects_color_and_no_color_together() {
        let err = Cli::try_parse_from(["terminal-weather", "--color", "always", "--no-color"])
            .expect_err("expected conflict");
        let rendered = err.to_string();
        assert!(rendered.contains("--color"));
        assert!(rendered.contains("--no-color"));
    }

    #[test]
    fn effective_color_mode_prefers_no_color() {
        let cli = Cli::parse_from(["terminal-weather", "--no-color"]);
        assert_eq!(cli.effective_color_mode(), ColorArg::Never);

        let cli = Cli::parse_from(["terminal-weather", "--color", "never"]);
        assert_eq!(cli.effective_color_mode(), ColorArg::Never);

        let cli = Cli::parse_from(["terminal-weather", "--color", "always"]);
        assert_eq!(cli.effective_color_mode(), ColorArg::Always);

        let cli = Cli::parse_from(["terminal-weather"]);
        assert_eq!(cli.effective_color_mode(), ColorArg::Auto);
    }

    #[test]
    fn effective_motion_mode_prefers_explicit_motion_flag() {
        let cli = Cli::parse_from([
            "terminal-weather",
            "--no-animation",
            "--motion",
            "cinematic",
        ]);
        assert_eq!(cli.effective_motion_mode(), MotionMode::Cinematic);

        let cli = Cli::parse_from(["terminal-weather", "--reduced-motion"]);
        assert_eq!(cli.effective_motion_mode(), MotionMode::Reduced);

        let cli = Cli::parse_from(["terminal-weather", "--no-animation"]);
        assert_eq!(cli.effective_motion_mode(), MotionMode::Off);
    }

    #[test]
    fn parses_hourly_view_override() {
        let cli = Cli::parse_from(["terminal-weather", "--hourly-view", "hybrid"]);
        assert_eq!(cli.hourly_view, Some(HourlyViewArg::Hybrid));
    }

    #[test]
    fn validate_rejects_invalid_lat() {
        let cli = Cli {
            lat: Some(91.0),
            lon: Some(0.0),
            ..Cli::parse_from(["terminal-weather"])
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_lon() {
        let cli = Cli {
            lat: Some(0.0),
            lon: Some(181.0),
            ..Cli::parse_from(["terminal-weather"])
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn validate_accepts_valid_coords() {
        let cli = Cli {
            lat: Some(45.0),
            lon: Some(-90.0),
            ..Cli::parse_from(["terminal-weather"])
        };
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn parses_new_theme_variants() {
        let cli = Cli::parse_from(["terminal-weather", "--theme", "tokyo-night-storm"]);
        assert_eq!(cli.theme, super::ThemeArg::TokyoNightStorm);

        let cli = Cli::parse_from(["terminal-weather", "--theme", "rose-pine-moon"]);
        assert_eq!(cli.theme, super::ThemeArg::RosePineMoon);

        let cli = Cli::parse_from(["terminal-weather", "--theme", "everforest-dark"]);
        assert_eq!(cli.theme, super::ThemeArg::EverforestDark);
    }
}

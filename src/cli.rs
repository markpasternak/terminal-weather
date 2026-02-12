use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum UnitsArg {
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconMode {
    Unicode,
    Ascii,
    Emoji,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeArg {
    Auto,
    Aurora,
    MidnightCyan,
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

#[derive(Debug, Parser, Clone)]
#[command(
    name = "atmos-tui",
    version,
    about = "Animated terminal weather dashboard"
)]
pub struct Cli {
    /// City name (default: Stockholm)
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

    /// Disable thunder flash
    #[arg(long)]
    pub no_flash: bool,

    /// Force ASCII icons
    #[arg(long)]
    pub ascii_icons: bool,

    /// Force emoji icons
    #[arg(long)]
    pub emoji_icons: bool,

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

    /// Refresh interval in seconds
    #[arg(long, default_value_t = 600)]
    pub refresh_interval: u64,
}

impl Cli {
    pub fn default_city(&self) -> String {
        self.city.clone().unwrap_or_else(|| "Stockholm".to_string())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        match (self.lat, self.lon) {
            (Some(_), None) | (None, Some(_)) => {
                anyhow::bail!("--lat and --lon must be provided together")
            }
            _ => Ok(()),
        }
    }
}

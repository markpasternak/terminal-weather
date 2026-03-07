mod palette;
mod signal;

use chrono::NaiveDateTime;

use crate::domain::weather::{ForecastBundle, HourlyForecast, WeatherCategory};

use super::{ColorCapability, Rgb, Theme, extended::theme_for_palette};
use palette::{auto_theme_palette, blend_auto_palette};
use signal::{auto_theme_signal, fog_now, rain_now_or_soon, snow_now_or_soon};

const AUTO_PHASE_LABELS: [&str; 8] = [
    "pre-dawn",
    "sunrise",
    "morning",
    "solar noon",
    "late afternoon",
    "golden hour",
    "blue hour",
    "deep night",
];

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AutoPhase {
    PreDawn,
    Sunrise,
    Morning,
    SolarNoon,
    LateAfternoon,
    GoldenHour,
    BlueHour,
    DeepNight,
}

impl AutoPhase {
    #[must_use]
    pub(super) const fn label(self) -> &'static str {
        AUTO_PHASE_LABELS[self as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AutoPaletteAnchor {
    pub phase: AutoPhase,
    pub minutes: u16,
    pub top: Rgb,
    pub bottom: Rgb,
    pub accent: Rgb,
    pub ambient: Rgb,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AutoThemeSignal<'a> {
    pub now_local: NaiveDateTime,
    pub sunrise_today: Option<NaiveDateTime>,
    pub sunset_today: Option<NaiveDateTime>,
    pub current_category: WeatherCategory,
    pub current_cloud_cover: f32,
    pub current_visibility_m: f32,
    pub current_precip_mm: f32,
    pub next_6h: &'a [HourlyForecast],
    pub incoming_category: WeatherCategory,
    pub max_precip_mm_6h: f32,
    pub max_snow_cm_6h: f32,
    pub max_cloud_cover_6h: f32,
    pub thunder_soon: bool,
    pub strong_wind_soon: bool,
    pub clearing_soon: bool,
}

pub(super) fn auto_theme_from_bundle(
    bundle: &ForecastBundle,
    capability: ColorCapability,
) -> Option<Theme> {
    let signal = auto_theme_signal(bundle)?;
    Some(theme_for_palette(auto_theme_palette(&signal), capability))
}

pub(super) fn auto_theme_preview(bundle: &ForecastBundle) -> Option<String> {
    let signal = auto_theme_signal(bundle)?;
    let phase = blend_auto_palette(&signal).phase.label();
    Some(format!("Auto: {}", preview_summary(&signal, phase)))
}

fn preview_summary(signal: &AutoThemeSignal<'_>, phase: &str) -> String {
    priority_preview_summary(signal, phase)
        .unwrap_or_else(|| weather_preview_summary(signal.current_category, phase))
}

fn priority_preview_summary(signal: &AutoThemeSignal<'_>, phase: &str) -> Option<String> {
    if signal.clearing_soon {
        return Some(format!("{phase} clearing"));
    }
    if storm_pressure_building(signal) {
        return Some(format!("{phase} storm pressure building"));
    }
    if snow_now_or_soon(signal) {
        return Some(format!("{phase} snow glow"));
    }
    if rain_now_or_soon(signal) {
        return Some(format!("{phase} rain moving in"));
    }
    if fog_now(signal) {
        return Some(format!("{phase} low-visibility mist"));
    }
    None
}

fn storm_pressure_building(signal: &AutoThemeSignal<'_>) -> bool {
    signal.thunder_soon || signal.strong_wind_soon
}

fn weather_preview_summary(category: WeatherCategory, phase: &str) -> String {
    match category {
        WeatherCategory::Clear => format!("{phase}, crisp and clear"),
        WeatherCategory::Cloudy => format!("{phase} overcast calm"),
        _ => format!("{phase} weather-aware palette"),
    }
}

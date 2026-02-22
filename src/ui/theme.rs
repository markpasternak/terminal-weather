#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::manual_midpoint,
    clippy::match_same_arms,
    clippy::must_use_candidate
)]

use ratatui::style::Color;

use crate::{
    app::state::AppState,
    cli::{ColorArg, ThemeArg},
    domain::weather::WeatherCategory,
};

mod basic16;
mod capability;
mod contrast;
mod data;
mod extended;
mod quantize;
mod resolve;

#[cfg(test)]
use basic16::auto_basic16_gradient;
#[cfg(test)]
use capability::detect_color_capability_from;
#[cfg(test)]
use contrast::{contrast_ratio, min_contrast_ratio, relative_luminance};
use contrast::{ensure_contrast, ensure_contrast_multi, luma, mix_rgb};
pub use quantize::quantize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Xterm256,
    Basic16,
}

type Rgb = (u8, u8, u8);
type ThemeSeed = (Rgb, Rgb, Rgb);
type Basic16Palette = (Color, Color, Color, Color, Color, Color);

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub top: Color,
    pub bottom: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub popup_surface: Color,
    pub accent: Color,
    pub text: Color,
    pub muted_text: Color,
    pub popup_text: Color,
    pub popup_muted_text: Color,
    pub particle: Color,
    pub border: Color,
    pub popup_border: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub temp_freezing: Color,
    pub temp_cold: Color,
    pub temp_mild: Color,
    pub temp_warm: Color,
    pub temp_hot: Color,
    pub range_track: Color,
    pub landmark_warm: Color,
    pub landmark_cool: Color,
    pub landmark_neutral: Color,
}

pub fn detect_color_capability(mode: ColorArg) -> ColorCapability {
    resolve::detect_color_capability(mode)
}

pub fn resolved_theme(state: &AppState) -> Theme {
    resolve::resolved_theme(state)
}

pub fn theme_for(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
    mode: ThemeArg,
) -> Theme {
    resolve::theme_for(category, is_day, capability, mode)
}

pub fn condition_color(theme: &Theme, category: WeatherCategory) -> Color {
    match category {
        WeatherCategory::Clear => theme.warning,
        WeatherCategory::Cloudy => theme.muted_text,
        WeatherCategory::Rain => theme.info,
        WeatherCategory::Snow => theme.text,
        WeatherCategory::Fog => theme.landmark_neutral,
        WeatherCategory::Thunder => theme.danger,
        WeatherCategory::Unknown => theme.accent,
    }
}

pub fn icon_color(theme: &Theme, category: WeatherCategory) -> Color {
    match category {
        WeatherCategory::Clear => theme.warning,
        WeatherCategory::Cloudy => theme.muted_text,
        WeatherCategory::Rain => theme.info,
        WeatherCategory::Snow => theme.text,
        WeatherCategory::Fog => theme.landmark_neutral,
        WeatherCategory::Thunder => theme.danger,
        WeatherCategory::Unknown => theme.accent,
    }
}

pub fn temp_color(theme: &Theme, temp: f32) -> Color {
    if temp <= -8.0 {
        theme.temp_freezing
    } else if temp <= 2.0 {
        theme.temp_cold
    } else if temp <= 16.0 {
        theme.temp_mild
    } else if temp <= 28.0 {
        theme.temp_warm
    } else {
        theme.temp_hot
    }
}

#[cfg(test)]
mod tests;

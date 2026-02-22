use ratatui::style::Color;

use crate::{cli::ThemeArg, domain::weather::WeatherCategory};

use super::data::BASIC16_MODE_PALETTES;
use super::extended::quantize_rgb;
use super::resolve::{auto_theme_seed, lookup_theme_entry};
use super::{ColorCapability, Rgb, Theme};

pub(super) fn theme_for_basic16(
    mode: ThemeArg,
    category: WeatherCategory,
    is_day: bool,
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    capability: ColorCapability,
) -> Theme {
    if mode == ThemeArg::Auto {
        return auto_basic16_theme(category, is_day, capability);
    }
    explicit_basic16_theme(mode, top, bottom, capability)
}

#[derive(Debug, Clone, Copy)]
struct Basic16Semantics {
    text: Color,
    muted: Color,
    particle: Color,
    info: Color,
    success: Color,
    warning: Color,
    danger: Color,
    temp_freezing: Color,
    temp_cold: Color,
    temp_mild: Color,
    temp_warm: Color,
    temp_hot: Color,
    landmark_warm: Color,
    landmark_cool: Color,
}

pub(super) fn auto_basic16_gradient(
    category: WeatherCategory,
    is_day: bool,
) -> ((u8, u8, u8), (u8, u8, u8)) {
    let (top, bottom, _) = auto_theme_seed(category, is_day);
    (top, bottom)
}

fn basic16_mode_palette(mode: ThemeArg) -> (Color, Color, Color, Color, Color, Color) {
    lookup_theme_entry(BASIC16_MODE_PALETTES, mode)
}

const BASIC16_DARK_SEMANTICS: Basic16Semantics = Basic16Semantics {
    text: Color::White,
    muted: Color::Gray,
    particle: Color::DarkGray,
    info: Color::LightCyan,
    success: Color::LightGreen,
    warning: Color::Yellow,
    danger: Color::LightRed,
    temp_freezing: Color::LightBlue,
    temp_cold: Color::Cyan,
    temp_mild: Color::Green,
    temp_warm: Color::Yellow,
    temp_hot: Color::LightRed,
    landmark_warm: Color::Yellow,
    landmark_cool: Color::LightBlue,
};

const BASIC16_LIGHT_SEMANTICS: Basic16Semantics = Basic16Semantics {
    text: Color::Black,
    muted: Color::DarkGray,
    particle: Color::Gray,
    info: Color::Blue,
    success: Color::Green,
    warning: Color::Magenta,
    danger: Color::Red,
    temp_freezing: Color::Blue,
    temp_cold: Color::Cyan,
    temp_mild: Color::Green,
    temp_warm: Color::Magenta,
    temp_hot: Color::Red,
    landmark_warm: Color::Magenta,
    landmark_cool: Color::Blue,
};

fn basic16_semantics(is_light_theme: bool) -> Basic16Semantics {
    if is_light_theme {
        BASIC16_LIGHT_SEMANTICS
    } else {
        BASIC16_DARK_SEMANTICS
    }
}

fn auto_basic16_theme(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
) -> Theme {
    let (top, bottom) = auto_basic16_gradient(category, is_day);
    Theme {
        top: quantize_rgb(top, capability),
        bottom: quantize_rgb(bottom, capability),
        surface: Color::Black,
        surface_alt: Color::DarkGray,
        popup_surface: Color::Blue,
        accent: if is_day {
            Color::Cyan
        } else {
            Color::LightBlue
        },
        text: Color::White,
        muted_text: Color::Gray,
        popup_text: Color::White,
        popup_muted_text: Color::Gray,
        particle: Color::Gray,
        border: Color::LightCyan,
        popup_border: Color::Yellow,
        info: Color::LightCyan,
        success: Color::LightGreen,
        warning: Color::Yellow,
        danger: Color::LightRed,
        temp_freezing: Color::LightBlue,
        temp_cold: Color::Cyan,
        temp_mild: Color::Green,
        temp_warm: Color::Yellow,
        temp_hot: Color::LightRed,
        range_track: Color::Gray,
        landmark_warm: Color::Yellow,
        landmark_cool: Color::LightBlue,
        landmark_neutral: Color::Gray,
    }
}

fn explicit_basic16_theme(
    mode: ThemeArg,
    top: Rgb,
    bottom: Rgb,
    capability: ColorCapability,
) -> Theme {
    let (surface, surface_alt, popup_surface, accent, border, popup_border) =
        basic16_mode_palette(mode);
    let semantics = basic16_semantics(matches!(mode, ThemeArg::AyuLight | ThemeArg::Hoth));
    Theme {
        top: quantize_rgb(top, capability),
        bottom: quantize_rgb(bottom, capability),
        surface,
        surface_alt,
        popup_surface,
        accent,
        text: semantics.text,
        muted_text: semantics.muted,
        popup_text: semantics.text,
        popup_muted_text: semantics.muted,
        particle: semantics.particle,
        border,
        popup_border,
        info: semantics.info,
        success: semantics.success,
        warning: semantics.warning,
        danger: semantics.danger,
        temp_freezing: semantics.temp_freezing,
        temp_cold: semantics.temp_cold,
        temp_mild: semantics.temp_mild,
        temp_warm: semantics.temp_warm,
        temp_hot: semantics.temp_hot,
        range_track: semantics.muted,
        landmark_warm: semantics.landmark_warm,
        landmark_cool: semantics.landmark_cool,
        landmark_neutral: semantics.muted,
    }
}

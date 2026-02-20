#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::manual_midpoint,
    clippy::match_same_arms,
    clippy::must_use_candidate,
    clippy::too_many_lines
)]

use ratatui::style::Color;

use crate::{
    cli::{ColorArg, ThemeArg},
    domain::weather::WeatherCategory,
};

mod capability;
mod data;
mod extended;

use capability::detect_color_capability_from;
use data::{AUTO_THEME_SEEDS, BASIC16_MODE_PALETTES, PRESET_THEME_SEEDS};
use extended::{quantize_rgb, theme_for_extended};

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
    let term = std::env::var("TERM").ok();
    let colorterm = std::env::var("COLORTERM").ok();
    let no_color = std::env::var("NO_COLOR").ok();
    detect_color_capability_from(
        mode,
        term.as_deref(),
        colorterm.as_deref(),
        no_color.as_deref(),
    )
}

pub fn theme_for(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
    mode: ThemeArg,
) -> Theme {
    let (top, bottom, accent_seed) = match mode {
        ThemeArg::Auto => auto_theme_seed(category, is_day),
        _ => preset_theme_seed(mode),
    };

    if capability == ColorCapability::Basic16 {
        return theme_for_basic16(mode, category, top, bottom, capability);
    }

    theme_for_extended(top, bottom, accent_seed, capability)
}

fn auto_theme_seed(category: WeatherCategory, is_day: bool) -> ThemeSeed {
    for ((candidate_category, candidate_is_day), seed) in AUTO_THEME_SEEDS {
        if *candidate_category == category && *candidate_is_day == is_day {
            return *seed;
        }
    }
    ((28, 36, 51), (42, 53, 73), (205, 219, 234))
}

fn lookup_theme_entry<T: Copy>(entries: &[(ThemeArg, T)], mode: ThemeArg) -> T {
    debug_assert!(mode != ThemeArg::Auto, "auto mode handled separately");
    for (candidate, value) in entries {
        if *candidate == mode {
            return *value;
        }
    }
    unreachable!("missing theme mapping for {:?}", mode)
}

fn preset_theme_seed(mode: ThemeArg) -> ThemeSeed {
    lookup_theme_entry(PRESET_THEME_SEEDS, mode)
}

fn theme_for_basic16(
    mode: ThemeArg,
    category: WeatherCategory,
    top: (u8, u8, u8),
    bottom: (u8, u8, u8),
    capability: ColorCapability,
) -> Theme {
    if mode == ThemeArg::Auto {
        return auto_basic16_theme(category, capability);
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

fn auto_basic16_gradient(category: WeatherCategory) -> ((u8, u8, u8), (u8, u8, u8)) {
    let top = (0, 0, 0);
    let bottom = match category {
        WeatherCategory::Clear => (0, 32, 72),
        WeatherCategory::Cloudy => (25, 30, 35),
        WeatherCategory::Rain => (0, 22, 56),
        WeatherCategory::Snow => (28, 38, 56),
        WeatherCategory::Fog => (30, 30, 30),
        WeatherCategory::Thunder => (24, 0, 44),
        WeatherCategory::Unknown => (20, 24, 32),
    };
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

fn auto_basic16_theme(category: WeatherCategory, capability: ColorCapability) -> Theme {
    let (top, bottom) = auto_basic16_gradient(category);
    Theme {
        top: quantize_rgb(top, capability),
        bottom: quantize_rgb(bottom, capability),
        surface: Color::Black,
        surface_alt: Color::DarkGray,
        popup_surface: Color::Blue,
        accent: Color::Cyan,
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

fn luma(r: u8, g: u8, b: u8) -> f32 {
    (0.2126 * f32::from(r)) + (0.7152 * f32::from(g)) + (0.0722 * f32::from(b))
}

fn mix_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn ensure_contrast(fg: (u8, u8, u8), bg: (u8, u8, u8), min_ratio: f32) -> (u8, u8, u8) {
    ensure_contrast_multi(fg, &[bg], min_ratio)
}

fn ensure_contrast_multi(
    fg: (u8, u8, u8),
    backgrounds: &[(u8, u8, u8)],
    min_ratio: f32,
) -> (u8, u8, u8) {
    if backgrounds.is_empty() {
        return fg;
    }
    if min_contrast_ratio(fg, backgrounds) >= min_ratio {
        return fg;
    }

    let black = (0, 0, 0);
    let white = (255, 255, 255);
    let black_score = min_contrast_ratio(black, backgrounds);
    let white_score = min_contrast_ratio(white, backgrounds);
    let target = if white_score >= black_score {
        white
    } else {
        black
    };

    let mut best = fg;
    let mut best_ratio = min_contrast_ratio(fg, backgrounds);
    for step in 1..=24 {
        let t = step as f32 / 24.0;
        let candidate = mix_rgb(fg, target, t);
        let ratio = min_contrast_ratio(candidate, backgrounds);
        if ratio > best_ratio {
            best = candidate;
            best_ratio = ratio;
        }
        if ratio >= min_ratio {
            return candidate;
        }
    }
    best
}

fn min_contrast_ratio(color: (u8, u8, u8), backgrounds: &[(u8, u8, u8)]) -> f32 {
    backgrounds
        .iter()
        .map(|bg| contrast_ratio(color, *bg))
        .fold(f32::INFINITY, f32::min)
}

fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

fn relative_luminance(rgb: (u8, u8, u8)) -> f32 {
    let r = srgb_to_linear(rgb.0);
    let g = srgb_to_linear(rgb.1);
    let b = srgb_to_linear(rgb.2);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(v: u8) -> f32 {
    let s = f32::from(v) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

pub fn quantize(color: Color, capability: ColorCapability) -> Color {
    match (capability, color) {
        (ColorCapability::TrueColor, c) => c,
        (ColorCapability::Xterm256, Color::Rgb(r, g, b)) => {
            let to_cube = |v: u8| -> u8 { ((f32::from(v) / 255.0) * 5.0).round() as u8 };
            let ri = to_cube(r);
            let gi = to_cube(g);
            let bi = to_cube(b);
            let index = 16 + 36 * ri + 6 * gi + bi;
            Color::Indexed(index)
        }
        (ColorCapability::Basic16, Color::Rgb(r, g, b)) => basic16_from_rgb(r, g, b),
        (_, c) => c,
    }
}

fn basic16_from_rgb(r: u8, g: u8, b: u8) -> Color {
    let rf = f32::from(r) / 255.0;
    let gf = f32::from(g) / 255.0;
    let bf = f32::from(b) / 255.0;

    let max = rf.max(gf.max(bf));
    let min = rf.min(gf.min(bf));
    let delta = max - min;
    let light = (max + min) / 2.0;

    if delta < 0.08 {
        return achromatic_basic16(light);
    }

    let hue = hue_from_rgb_components(rf, gf, bf, max, delta);
    hue_to_basic16(hue, light >= 0.55)
}

fn achromatic_basic16(light: f32) -> Color {
    if light < 0.20 {
        return Color::Black;
    }
    if light < 0.40 {
        return Color::DarkGray;
    }
    if light < 0.72 {
        return Color::Gray;
    }
    Color::White
}

fn hue_from_rgb_components(rf: f32, gf: f32, bf: f32, max: f32, delta: f32) -> f32 {
    if (max - rf).abs() < f32::EPSILON {
        return 60.0 * ((gf - bf) / delta).rem_euclid(6.0);
    }
    if (max - gf).abs() < f32::EPSILON {
        return 60.0 * (((bf - rf) / delta) + 2.0);
    }
    60.0 * (((rf - gf) / delta) + 4.0)
}

fn hue_to_basic16(hue: f32, bright: bool) -> Color {
    let band = if !(30.0..330.0).contains(&hue) {
        0
    } else if hue < 90.0 {
        1
    } else if hue < 150.0 {
        2
    } else if hue < 210.0 {
        3
    } else if hue < 270.0 {
        4
    } else {
        5
    };
    hue_band_color(band, bright)
}

fn hue_band_color(band: usize, bright: bool) -> Color {
    const DIM: [Color; 6] = [
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
    ];
    const BRIGHT: [Color; 6] = [
        Color::LightRed,
        Color::LightYellow,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightBlue,
        Color::LightMagenta,
    ];
    if bright { BRIGHT[band] } else { DIM[band] }
}

#[cfg(test)]
mod tests;

use crate::{cli::ThemeArg, domain::weather::WeatherCategory};

use super::{Basic16Palette, Rgb, ThemeSeed};

mod specs;

#[cfg(test)]
pub(super) const ALL_NON_AUTO_THEMES: &[ThemeArg] = specs::ALL_NON_AUTO_THEMES;

pub(super) fn theme_specs() -> impl Iterator<Item = &'static ThemeSpec> {
    specs::theme_specs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ThemeAppearance {
    Dark,
    Light,
}

impl ThemeAppearance {
    #[must_use]
    pub(super) const fn is_light(self) -> bool {
        matches!(self, Self::Light)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(super) struct ThemeSpec {
    pub mode: ThemeArg,
    pub label: &'static str,
    pub family: &'static str,
    pub variant: &'static str,
    pub appearance: ThemeAppearance,
    pub source_url: Option<&'static str>,
    pub mood_note: &'static str,
    pub top: Rgb,
    pub bottom: Rgb,
    pub surface: Rgb,
    pub surface_alt: Rgb,
    pub popup_surface: Rgb,
    pub accent: Rgb,
    pub text_hint: Rgb,
    pub muted_hint: Rgb,
    pub info_seed: Rgb,
    pub success_seed: Rgb,
    pub warning_seed: Rgb,
    pub danger_seed: Rgb,
    pub landmark_warm_seed: Rgb,
    pub landmark_cool_seed: Rgb,
    pub landmark_neutral_seed: Rgb,
    pub temp_freezing_seed: Rgb,
    pub temp_cold_seed: Rgb,
    pub temp_mild_seed: Rgb,
    pub temp_warm_seed: Rgb,
    pub temp_hot_seed: Rgb,
    pub basic16: Basic16Palette,
}

pub(super) const AUTO_THEME_SEEDS: &[((WeatherCategory, bool), ThemeSeed)] = &[
    (
        (WeatherCategory::Clear, true),
        ((13, 53, 102), (30, 102, 158), (255, 215, 117)),
    ),
    (
        (WeatherCategory::Clear, false),
        ((9, 18, 44), (21, 43, 79), (173, 216, 255)),
    ),
    (
        (WeatherCategory::Cloudy, true),
        ((25, 36, 51), (48, 63, 84), (210, 223, 235)),
    ),
    (
        (WeatherCategory::Cloudy, false),
        ((20, 26, 40), (34, 42, 62), (194, 207, 224)),
    ),
    (
        (WeatherCategory::Rain, true),
        ((17, 47, 88), (32, 73, 126), (153, 214, 255)),
    ),
    (
        (WeatherCategory::Rain, false),
        ((12, 25, 52), (25, 44, 78), (143, 196, 255)),
    ),
    (
        (WeatherCategory::Snow, true),
        ((27, 51, 77), (43, 74, 106), (237, 247, 255)),
    ),
    (
        (WeatherCategory::Snow, false),
        ((19, 35, 55), (34, 55, 80), (226, 241, 255)),
    ),
    (
        (WeatherCategory::Fog, true),
        ((30, 34, 40), (50, 55, 62), (216, 220, 224)),
    ),
    (
        (WeatherCategory::Fog, false),
        ((21, 24, 30), (33, 37, 43), (201, 207, 211)),
    ),
    (
        (WeatherCategory::Thunder, true),
        ((28, 25, 66), (42, 40, 97), (255, 223, 112)),
    ),
    (
        (WeatherCategory::Thunder, false),
        ((18, 15, 44), (28, 24, 63), (255, 208, 95)),
    ),
    (
        (WeatherCategory::Unknown, true),
        ((28, 36, 51), (42, 53, 73), (205, 219, 234)),
    ),
    (
        (WeatherCategory::Unknown, false),
        ((19, 24, 35), (31, 39, 53), (195, 205, 215)),
    ),
];

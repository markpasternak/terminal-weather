use crate::{
    app::state::AppState,
    cli::{ColorArg, ThemeArg},
    domain::weather::{WeatherCategory, weather_code_to_category},
};

use super::auto::{auto_theme_from_bundle, auto_theme_preview};
use super::basic16::theme_for_basic16;
use super::data::{AUTO_THEME_SEEDS, ThemeSpec, theme_specs};
use super::extended::{ThemePalette, theme_for_palette};
use super::{ColorCapability, Theme, ThemeSeed};

pub(super) fn detect_color_capability(mode: ColorArg) -> ColorCapability {
    let term = std::env::var("TERM").ok();
    let colorterm = std::env::var("COLORTERM").ok();
    let no_color = std::env::var("NO_COLOR").ok();
    super::capability::detect_color_capability_from(
        mode,
        term.as_deref(),
        colorterm.as_deref(),
        no_color.as_deref(),
    )
}

pub(super) fn resolved_theme(state: &AppState) -> Theme {
    let capability = detect_color_capability(state.color_mode);
    match (state.settings.theme, state.weather.as_ref()) {
        (ThemeArg::Auto, Some(bundle)) => auto_theme_from_bundle(bundle, capability)
            .unwrap_or_else(|| fallback_auto_theme(capability)),
        (ThemeArg::Auto, None) => fallback_auto_theme(capability),
        (mode, _) => explicit_theme(mode, capability),
    }
}

pub(super) fn theme_preview(state: &AppState) -> String {
    match (state.settings.theme, state.weather.as_ref()) {
        (ThemeArg::Auto, Some(bundle)) => auto_theme_preview(bundle)
            .unwrap_or_else(|| "Auto: cinematic weather-aware palette".to_string()),
        (ThemeArg::Auto, None) => "Auto: cinematic weather-aware palette".to_string(),
        (mode, _) => {
            let spec = theme_spec(mode);
            format!("{}: {}", spec.label, spec.mood_note)
        }
    }
}

pub(super) fn theme_for(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
    mode: ThemeArg,
) -> Theme {
    match mode {
        ThemeArg::Auto => coarse_auto_theme(category, is_day, capability),
        _ => explicit_theme(mode, capability),
    }
}

pub(super) fn auto_theme_seed(category: WeatherCategory, is_day: bool) -> ThemeSeed {
    for ((candidate_category, candidate_is_day), seed) in AUTO_THEME_SEEDS {
        if *candidate_category == category && *candidate_is_day == is_day {
            return *seed;
        }
    }
    ((28, 36, 51), (42, 53, 73), (205, 219, 234))
}

pub(super) fn theme_spec(mode: ThemeArg) -> &'static ThemeSpec {
    for spec in theme_specs() {
        if spec.mode == mode {
            return spec;
        }
    }
    unreachable!("missing theme spec for {:?}", mode)
}

fn explicit_theme(mode: ThemeArg, capability: ColorCapability) -> Theme {
    if capability == ColorCapability::Basic16 {
        return theme_for_basic16(mode, WeatherCategory::Unknown, false, capability);
    }
    theme_for_palette(palette_from_spec(theme_spec(mode)), capability)
}

fn palette_from_spec(spec: &ThemeSpec) -> ThemePalette {
    ThemePalette {
        appearance: spec.appearance,
        top: spec.top,
        bottom: spec.bottom,
        surface: spec.surface,
        surface_alt: spec.surface_alt,
        popup_surface: spec.popup_surface,
        accent: spec.accent,
        text_hint: spec.text_hint,
        muted_hint: spec.muted_hint,
        info: spec.info_seed,
        success: spec.success_seed,
        warning: spec.warning_seed,
        danger: spec.danger_seed,
        temp_freezing: spec.temp_freezing_seed,
        temp_cold: spec.temp_cold_seed,
        temp_mild: spec.temp_mild_seed,
        temp_warm: spec.temp_warm_seed,
        temp_hot: spec.temp_hot_seed,
        landmark_warm: spec.landmark_warm_seed,
        landmark_cool: spec.landmark_cool_seed,
        landmark_neutral: spec.landmark_neutral_seed,
        particle: None,
        border: None,
        popup_border: None,
        range_track: None,
    }
}

fn coarse_auto_theme(
    category: WeatherCategory,
    is_day: bool,
    capability: ColorCapability,
) -> Theme {
    let (top, bottom, accent) = auto_theme_seed(category, is_day);
    if capability == ColorCapability::Basic16 {
        return theme_for_basic16(ThemeArg::Auto, category, is_day, capability);
    }

    let current_category = normalized_auto_category(category);
    let ambient = ambient_color(current_category, is_day);
    let appearance = auto_appearance(current_category, is_day);
    let surface = super::mix_rgb(top, bottom, 0.78);
    let surface_alt = super::mix_rgb(top, bottom, 0.60);
    let popup_surface = auto_popup_surface(surface_alt, appearance);

    theme_for_palette(
        ThemePalette {
            appearance,
            top,
            bottom,
            surface: super::mix_rgb(surface, ambient, 0.16),
            surface_alt: super::mix_rgb(surface_alt, ambient, 0.20),
            popup_surface,
            accent,
            text_hint: auto_text_hint(appearance),
            muted_hint: auto_muted_hint(appearance),
            info: (97, 176, 255),
            success: (113, 219, 165),
            warning: (255, 196, 103),
            danger: (243, 113, 130),
            temp_freezing: (160, 212, 255),
            temp_cold: (102, 194, 255),
            temp_mild: (120, 223, 172),
            temp_warm: (255, 200, 105),
            temp_hot: (247, 125, 112),
            landmark_warm: super::mix_rgb((255, 220, 152), accent, 0.20),
            landmark_cool: super::mix_rgb((154, 204, 255), ambient, 0.20),
            landmark_neutral: super::mix_rgb((168, 180, 195), ambient, 0.16),
            particle: Some(super::mix_rgb(ambient, accent, 0.22)),
            border: None,
            popup_border: None,
            range_track: None,
        },
        capability,
    )
}

fn fallback_auto_theme(capability: ColorCapability) -> Theme {
    coarse_auto_theme(WeatherCategory::Unknown, false, capability)
}

fn normalized_auto_category(category: WeatherCategory) -> WeatherCategory {
    if category == WeatherCategory::Unknown {
        weather_code_to_category(3)
    } else {
        category
    }
}

fn ambient_color(category: WeatherCategory, is_day: bool) -> (u8, u8, u8) {
    match category {
        WeatherCategory::Clear => clear_ambient(is_day),
        WeatherCategory::Cloudy => (120, 137, 158),
        WeatherCategory::Rain => (78, 123, 188),
        WeatherCategory::Snow => (168, 209, 241),
        WeatherCategory::Fog => (151, 160, 176),
        WeatherCategory::Thunder => (86, 74, 138),
        WeatherCategory::Unknown => (110, 131, 160),
    }
}

fn clear_ambient(is_day: bool) -> (u8, u8, u8) {
    if is_day {
        (112, 178, 215)
    } else {
        (70, 96, 150)
    }
}

fn auto_appearance(category: WeatherCategory, is_day: bool) -> super::data::ThemeAppearance {
    if is_day && matches!(category, WeatherCategory::Clear | WeatherCategory::Snow) {
        super::data::ThemeAppearance::Light
    } else {
        super::data::ThemeAppearance::Dark
    }
}

fn auto_popup_surface(
    surface_alt: (u8, u8, u8),
    appearance: super::data::ThemeAppearance,
) -> (u8, u8, u8) {
    if appearance.is_light() {
        super::mix_rgb(surface_alt, (245, 248, 252), 0.12)
    } else {
        super::mix_rgb(surface_alt, (12, 17, 28), 0.18)
    }
}

fn auto_text_hint(appearance: super::data::ThemeAppearance) -> (u8, u8, u8) {
    if appearance.is_light() {
        (15, 20, 30)
    } else {
        (236, 241, 248)
    }
}

fn auto_muted_hint(appearance: super::data::ThemeAppearance) -> (u8, u8, u8) {
    if appearance.is_light() {
        (77, 90, 109)
    } else {
        (174, 188, 205)
    }
}

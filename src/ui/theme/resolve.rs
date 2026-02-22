use crate::{
    app::state::AppState,
    cli::{ColorArg, ThemeArg},
    domain::weather::{WeatherCategory, weather_code_to_category},
};

use super::basic16::theme_for_basic16;
use super::data::{AUTO_THEME_SEEDS, PRESET_THEME_SEEDS};
use super::extended::theme_for_extended;
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
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    theme_for(category, is_day, capability, state.settings.theme)
}

pub(super) fn theme_for(
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
        return theme_for_basic16(mode, category, is_day, top, bottom, capability);
    }

    theme_for_extended(top, bottom, accent_seed, capability)
}

pub(super) fn auto_theme_seed(category: WeatherCategory, is_day: bool) -> ThemeSeed {
    for ((candidate_category, candidate_is_day), seed) in AUTO_THEME_SEEDS {
        if *candidate_category == category && *candidate_is_day == is_day {
            return *seed;
        }
    }
    ((28, 36, 51), (42, 53, 73), (205, 219, 234))
}

pub(super) fn lookup_theme_entry<T: Copy>(entries: &[(ThemeArg, T)], mode: ThemeArg) -> T {
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

mod built_in;
mod community;

use super::ThemeSpec;
#[cfg(test)]
use crate::cli::ThemeArg;

pub(super) fn theme_specs() -> impl Iterator<Item = &'static ThemeSpec> {
    built_in::THEME_SPECS_BUILT_IN
        .iter()
        .chain(community::THEME_SPECS_COMMUNITY.iter())
}

#[cfg(test)]
pub(super) const ALL_NON_AUTO_THEMES: &[ThemeArg] = &[
    ThemeArg::Aurora,
    ThemeArg::MidnightCyan,
    ThemeArg::Aubergine,
    ThemeArg::Hoth,
    ThemeArg::Monument,
    ThemeArg::Nord,
    ThemeArg::CatppuccinMocha,
    ThemeArg::Mono,
    ThemeArg::HighContrast,
    ThemeArg::Dracula,
    ThemeArg::GruvboxMaterialDark,
    ThemeArg::KanagawaWave,
    ThemeArg::AyuMirage,
    ThemeArg::AyuLight,
    ThemeArg::PoimandresStorm,
    ThemeArg::SelenizedDark,
    ThemeArg::NoClownFiesta,
    ThemeArg::TokyoNightStorm,
    ThemeArg::RosePineMoon,
    ThemeArg::EverforestDark,
];

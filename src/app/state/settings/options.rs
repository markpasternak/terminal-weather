use crate::cli::ThemeArg;
use crate::domain::weather::HourlyViewMode;

pub(crate) const REFRESH_OPTIONS: [u64; 4] = [300, 600, 900, 1800];
pub(crate) const HOURLY_VIEW_OPTIONS: [HourlyViewMode; 3] = [
    HourlyViewMode::Table,
    HourlyViewMode::Hybrid,
    HourlyViewMode::Chart,
];

const THEME_LABEL_TABLE: [(ThemeArg, &str); 21] = [
    (ThemeArg::Auto, "Auto"),
    (ThemeArg::Aurora, "Aurora"),
    (ThemeArg::MidnightCyan, "Midnight Cyan"),
    (ThemeArg::Aubergine, "Aubergine"),
    (ThemeArg::Hoth, "Hoth"),
    (ThemeArg::Monument, "Monument"),
    (ThemeArg::Nord, "Nord"),
    (ThemeArg::CatppuccinMocha, "Catppuccin Mocha"),
    (ThemeArg::Mono, "Mono"),
    (ThemeArg::HighContrast, "High contrast"),
    (ThemeArg::Dracula, "Dracula"),
    (ThemeArg::GruvboxMaterialDark, "Gruvbox Material"),
    (ThemeArg::KanagawaWave, "Kanagawa Wave"),
    (ThemeArg::AyuMirage, "Ayu Mirage"),
    (ThemeArg::AyuLight, "Ayu Light"),
    (ThemeArg::PoimandresStorm, "Poimandres Storm"),
    (ThemeArg::SelenizedDark, "Selenized Dark"),
    (ThemeArg::NoClownFiesta, "No Clown Fiesta"),
    (ThemeArg::TokyoNightStorm, "Tokyo Night Storm"),
    (ThemeArg::RosePineMoon, "Rose Pine Moon"),
    (ThemeArg::EverforestDark, "Everforest Dark"),
];

const fn theme_options() -> [ThemeArg; THEME_LABEL_TABLE.len()] {
    let mut options = [ThemeArg::Auto; THEME_LABEL_TABLE.len()];
    let mut idx = 0;
    while idx < THEME_LABEL_TABLE.len() {
        options[idx] = THEME_LABEL_TABLE[idx].0;
        idx += 1;
    }
    options
}

pub(crate) const THEME_OPTIONS: [ThemeArg; THEME_LABEL_TABLE.len()] = theme_options();

pub(super) const THEME_LABELS: &[(ThemeArg, &str)] = &THEME_LABEL_TABLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSelection {
    #[default]
    Theme,
    Motion,
    Flash,
    Icons,
    HeroVisual,
    InlineHints,
    CommandBar,
    HourlyView,
    Units,
    RefreshInterval,
    RefreshNow,
    Close,
}

pub(crate) const SETTINGS_ORDER: [SettingsSelection; 12] = [
    SettingsSelection::Theme,
    SettingsSelection::Motion,
    SettingsSelection::Flash,
    SettingsSelection::Icons,
    SettingsSelection::HeroVisual,
    SettingsSelection::InlineHints,
    SettingsSelection::CommandBar,
    SettingsSelection::HourlyView,
    SettingsSelection::Units,
    SettingsSelection::RefreshInterval,
    SettingsSelection::RefreshNow,
    SettingsSelection::Close,
];

impl SettingsSelection {
    #[must_use]
    pub fn next(&self) -> Self {
        let idx = selection_index(*self);
        SETTINGS_ORDER[(idx + 1).min(SETTINGS_ORDER.len() - 1)]
    }

    #[must_use]
    pub fn prev(&self) -> Self {
        let idx = selection_index(*self);
        SETTINGS_ORDER[idx.saturating_sub(1)]
    }

    #[must_use]
    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

pub(super) fn selection_index(selection: SettingsSelection) -> usize {
    SETTINGS_ORDER
        .iter()
        .position(|candidate| *candidate == selection)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_options_matches_theme_labels_order() {
        let options_from_labels = THEME_LABELS
            .iter()
            .map(|(theme, _)| *theme)
            .collect::<Vec<_>>();
        assert_eq!(theme_options().to_vec(), options_from_labels);
        assert_eq!(THEME_OPTIONS.to_vec(), options_from_labels);
    }
}

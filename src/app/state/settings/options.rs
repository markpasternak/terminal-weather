use crate::cli::ThemeArg;
use crate::domain::weather::HourlyViewMode;

pub(crate) const REFRESH_OPTIONS: [u64; 4] = [300, 600, 900, 1800];
pub(crate) const HOURLY_VIEW_OPTIONS: [HourlyViewMode; 3] = [
    HourlyViewMode::Table,
    HourlyViewMode::Hybrid,
    HourlyViewMode::Chart,
];

pub(crate) const THEME_OPTIONS: [ThemeArg; 18] = [
    ThemeArg::Auto,
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
];

pub(super) const THEME_LABELS: &[(ThemeArg, &str)] = &[
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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSelection {
    #[default]
    Units,
    Theme,
    Flash,
    Icons,
    InlineHints,
    CommandBar,
    HourlyView,
    HeroVisual,
    RefreshInterval,
    RefreshNow,
    Close,
}

pub(super) const SETTINGS_ORDER: [SettingsSelection; 11] = [
    SettingsSelection::Units,
    SettingsSelection::Theme,
    SettingsSelection::Flash,
    SettingsSelection::Icons,
    SettingsSelection::InlineHints,
    SettingsSelection::CommandBar,
    SettingsSelection::HourlyView,
    SettingsSelection::HeroVisual,
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

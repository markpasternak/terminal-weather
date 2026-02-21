use super::*;

pub(super) const REFRESH_OPTIONS: [u64; 4] = [300, 600, 900, 1800];
pub(super) const HOURLY_VIEW_OPTIONS: [HourlyViewMode; 3] = [
    HourlyViewMode::Table,
    HourlyViewMode::Hybrid,
    HourlyViewMode::Chart,
];

const THEME_OPTIONS: [ThemeArg; 18] = [
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

const THEME_LABELS: &[(ThemeArg, &str)] = &[
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
    Motion,
    Flash,
    Icons,
    HourlyView,
    HeroVisual,
    RefreshInterval,
    RefreshNow,
    Close,
}

impl SettingsSelection {
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::Units => Self::Theme,
            Self::Theme => Self::Motion,
            Self::Motion => Self::Flash,
            Self::Flash => Self::Icons,
            Self::Icons => Self::HourlyView,
            Self::HourlyView => Self::HeroVisual,
            Self::HeroVisual => Self::RefreshInterval,
            Self::RefreshInterval => Self::RefreshNow,
            Self::RefreshNow | Self::Close => Self::Close,
        }
    }

    #[must_use]
    pub fn prev(&self) -> Self {
        match self {
            Self::Units | Self::Theme => Self::Units,
            Self::Motion => Self::Theme,
            Self::Flash => Self::Motion,
            Self::Icons => Self::Flash,
            Self::HourlyView => Self::Icons,
            Self::HeroVisual => Self::HourlyView,
            Self::RefreshInterval => Self::HeroVisual,
            Self::RefreshNow => Self::RefreshInterval,
            Self::Close => Self::RefreshNow,
        }
    }

    #[must_use]
    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl AppState {
    #[must_use]
    pub fn settings_entries(&self) -> Vec<SettingsEntry> {
        vec![
            settings_entry("Units", units_name(self.settings.units), true),
            settings_entry("Theme", theme_name(self.settings.theme), true),
            settings_entry("Motion", motion_name(self.settings.motion), true),
            settings_entry(
                "Thunder Flash",
                if self.settings.no_flash { "Off" } else { "On" },
                true,
            ),
            settings_entry("Icons", icon_mode_name(self.settings.icon_mode), true),
            settings_entry("Hourly View", hourly_view_name(self.hourly_view_mode), true),
            settings_entry(
                "Hero Visual",
                hero_visual_name(self.settings.hero_visual),
                true,
            ),
            SettingsEntry {
                label: "Auto Refresh",
                value: format!("{} min", self.settings.refresh_interval_secs / 60),
                editable: true,
            },
            settings_entry("Action", "Refresh now", false),
            settings_entry("Panel", "Close", false),
        ]
    }

    #[must_use]
    pub fn settings_hint(&self) -> String {
        if self.settings_selected == SettingsSelection::HeroVisual {
            return hero_visual_hint(self.settings.hero_visual).to_string();
        }
        settings_hint_for_selection(self.settings_selected).to_string()
    }
}

pub(crate) fn adjust_setting_selection(
    state: &mut AppState,
    selection: SettingsSelection,
    direction: i8,
) -> bool {
    match selection {
        SettingsSelection::Units => adjust_cycle_setting(
            &mut state.settings.units,
            &[Units::Celsius, Units::Fahrenheit],
            direction,
        ),
        SettingsSelection::Theme => {
            adjust_cycle_setting(&mut state.settings.theme, &THEME_OPTIONS, direction)
        }
        SettingsSelection::Motion => adjust_cycle_setting(
            &mut state.settings.motion,
            &[
                MotionSetting::Full,
                MotionSetting::Reduced,
                MotionSetting::Off,
            ],
            direction,
        ),
        SettingsSelection::Flash => {
            state.settings.no_flash = !state.settings.no_flash;
            true
        }
        SettingsSelection::Icons => adjust_cycle_setting(
            &mut state.settings.icon_mode,
            &[IconMode::Unicode, IconMode::Ascii, IconMode::Emoji],
            direction,
        ),
        SettingsSelection::HourlyView => adjust_cycle_setting_from(
            &mut state.settings.hourly_view,
            state.hourly_view_mode,
            &HOURLY_VIEW_OPTIONS,
            direction,
        ),
        SettingsSelection::HeroVisual => adjust_cycle_setting(
            &mut state.settings.hero_visual,
            &[
                HeroVisualArg::AtmosCanvas,
                HeroVisualArg::GaugeCluster,
                HeroVisualArg::SkyObservatory,
            ],
            direction,
        ),
        SettingsSelection::RefreshInterval => adjust_cycle_setting(
            &mut state.settings.refresh_interval_secs,
            &REFRESH_OPTIONS,
            direction,
        ),
        SettingsSelection::RefreshNow | SettingsSelection::Close => false,
    }
}

pub(super) fn cycle<T: Copy + Eq>(values: &[T], current: T, direction: i8) -> T {
    if values.is_empty() {
        return current;
    }
    let idx = values.iter().position(|v| *v == current).unwrap_or(0);
    let len = values.len();
    let next = if direction >= 0 {
        (idx + 1) % len
    } else if idx == 0 {
        len - 1
    } else {
        idx - 1
    };
    values[next]
}

fn adjust_cycle_setting<T: Copy + Eq>(current: &mut T, values: &[T], direction: i8) -> bool {
    adjust_cycle_setting_from(current, *current, values, direction)
}

fn adjust_cycle_setting_from<T: Copy + Eq>(
    target: &mut T,
    current: T,
    values: &[T],
    direction: i8,
) -> bool {
    *target = cycle(values, current, direction);
    true
}

fn settings_entry(label: &'static str, value: &'static str, editable: bool) -> SettingsEntry {
    SettingsEntry {
        label,
        value: value.to_string(),
        editable,
    }
}

fn units_name(units: Units) -> &'static str {
    match units {
        Units::Celsius => "Celsius",
        Units::Fahrenheit => "Fahrenheit",
    }
}

fn theme_name(theme: ThemeArg) -> &'static str {
    for (candidate, label) in THEME_LABELS {
        if *candidate == theme {
            return label;
        }
    }
    "Auto"
}

fn motion_name(motion: MotionSetting) -> &'static str {
    match motion {
        MotionSetting::Full => "Full",
        MotionSetting::Reduced => "Reduced",
        MotionSetting::Off => "Off",
    }
}

fn icon_mode_name(mode: IconMode) -> &'static str {
    match mode {
        IconMode::Unicode => "Unicode",
        IconMode::Ascii => "ASCII",
        IconMode::Emoji => "Emoji",
    }
}

fn hourly_view_name(mode: HourlyViewMode) -> &'static str {
    match mode {
        HourlyViewMode::Table => "Table",
        HourlyViewMode::Hybrid => "Hybrid",
        HourlyViewMode::Chart => "Chart",
    }
}

fn hero_visual_name(mode: HeroVisualArg) -> &'static str {
    match mode {
        HeroVisualArg::AtmosCanvas => "Atmos Canvas",
        HeroVisualArg::GaugeCluster => "Gauge Cluster",
        HeroVisualArg::SkyObservatory => "Sky Observatory",
    }
}

fn hero_visual_hint(mode: HeroVisualArg) -> &'static str {
    match mode {
        HeroVisualArg::AtmosCanvas => {
            "Current panel right side: data-driven terrain + condition sky overlays"
        }
        HeroVisualArg::GaugeCluster => {
            "Current panel right side: live instrument panel (temp, humidity, wind, pressure, UV)"
        }
        HeroVisualArg::SkyObservatory => {
            "Current panel right side: sun/moon arc with weather strip and precipitation lane"
        }
    }
}

fn settings_hint_for_selection(selected: SettingsSelection) -> &'static str {
    match selected {
        SettingsSelection::Theme => {
            "Theme applies to all panels: Current, Hourly, 7-Day, popups, and status"
        }
        SettingsSelection::Motion => {
            "Motion controls the moving effects: weather particles + animated hero scene (Full/Reduced/Off)"
        }
        SettingsSelection::Icons => "Icon mode affects weather symbols in Hourly and 7-Day panels",
        SettingsSelection::HourlyView => {
            "Hourly View controls the Hourly panel: Table, Hybrid cards+charts, or Chart"
        }
        SettingsSelection::RefreshInterval => "Auto-refresh cadence updates immediately",
        _ => "Use left/right or Enter to change the selected setting",
    }
}

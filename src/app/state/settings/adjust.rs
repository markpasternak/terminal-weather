use super::super::AppState;
use super::super::SettingsEntry;
use super::options::{
    HOURLY_VIEW_OPTIONS, REFRESH_OPTIONS, SettingsSelection, THEME_LABELS, THEME_OPTIONS,
};
use crate::cli::{HeroVisualArg, IconMode, ThemeArg};
use crate::domain::weather::{HourlyViewMode, Units};

type SettingAdjuster = fn(&mut AppState, i8) -> bool;

const SETTING_ADJUSTERS: [(SettingsSelection, SettingAdjuster); 8] = [
    (SettingsSelection::Units, adjust_units_setting),
    (SettingsSelection::Theme, adjust_theme_setting),
    (SettingsSelection::Icons, adjust_icon_setting),
    (SettingsSelection::InlineHints, adjust_inline_hints_setting),
    (SettingsSelection::CommandBar, adjust_command_bar_setting),
    (SettingsSelection::HourlyView, adjust_hourly_view_setting),
    (SettingsSelection::HeroVisual, adjust_hero_visual_setting),
    (
        SettingsSelection::RefreshInterval,
        adjust_refresh_interval_setting,
    ),
];

pub(crate) fn adjust_setting_selection(
    state: &mut AppState,
    selection: SettingsSelection,
    direction: i8,
) -> bool {
    if matches!(
        selection,
        SettingsSelection::RefreshNow | SettingsSelection::Close
    ) {
        return false;
    }
    if selection == SettingsSelection::Flash {
        state.settings.no_flash = !state.settings.no_flash;
        return true;
    }
    apply_adjuster(state, selection, direction)
}

fn apply_adjuster(state: &mut AppState, selection: SettingsSelection, direction: i8) -> bool {
    for (candidate, adjuster) in SETTING_ADJUSTERS {
        if selection == candidate {
            return adjuster(state, direction);
        }
    }
    false
}

fn adjust_units_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting(
        &mut state.settings.units,
        &[Units::Celsius, Units::Fahrenheit],
        direction,
    )
}

fn adjust_theme_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting(&mut state.settings.theme, &THEME_OPTIONS, direction)
}

fn adjust_icon_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting(
        &mut state.settings.icon_mode,
        &[
            IconMode::Unicode,
            IconMode::Ascii,
            IconMode::Emoji,
            IconMode::NerdFont,
        ],
        direction,
    )
}

fn adjust_inline_hints_setting(state: &mut AppState, _direction: i8) -> bool {
    state.settings.inline_hints = !state.settings.inline_hints;
    true
}

fn adjust_command_bar_setting(state: &mut AppState, _direction: i8) -> bool {
    state.settings.command_bar_enabled = !state.settings.command_bar_enabled;
    if !state.settings.command_bar_enabled {
        state.command_bar.close();
    }
    true
}

fn adjust_hourly_view_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting_from(
        &mut state.settings.hourly_view,
        state.hourly_view_mode,
        &HOURLY_VIEW_OPTIONS,
        direction,
    )
}

fn adjust_hero_visual_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting(
        &mut state.settings.hero_visual,
        &[
            HeroVisualArg::AtmosCanvas,
            HeroVisualArg::GaugeCluster,
            HeroVisualArg::SkyObservatory,
        ],
        direction,
    )
}

fn adjust_refresh_interval_setting(state: &mut AppState, direction: i8) -> bool {
    adjust_cycle_setting(
        &mut state.settings.refresh_interval_secs,
        &REFRESH_OPTIONS,
        direction,
    )
}

pub(crate) fn cycle<T: Copy + Eq>(values: &[T], current: T, direction: i8) -> T {
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

fn theme_name(theme: ThemeArg) -> &'static str {
    for (candidate, label) in THEME_LABELS {
        if *candidate == theme {
            return label;
        }
    }
    "Auto"
}

fn icon_mode_name(mode: IconMode) -> &'static str {
    match mode {
        IconMode::Unicode => "Unicode",
        IconMode::Ascii => "ASCII",
        IconMode::Emoji => "Emoji",
        IconMode::NerdFont => "Nerd Font",
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
        SettingsSelection::Icons => {
            "Icon mode affects weather symbols. Nerd Font requires a patched font."
        }
        SettingsSelection::InlineHints => "Inline hints add local guidance in panels and overlays",
        SettingsSelection::CommandBar => {
            "Command Bar adds ':' shortcuts (city/theme/view/units/refresh/quit)"
        }
        SettingsSelection::HourlyView => {
            "Hourly View controls the Hourly panel: Table, Hybrid cards+charts, or Chart"
        }
        SettingsSelection::RefreshInterval => "Auto-refresh cadence updates immediately",
        _ => "Use left/right or Enter to change the selected setting",
    }
}

impl AppState {
    #[must_use]
    pub fn settings_entries(&self) -> Vec<SettingsEntry> {
        vec![
            settings_entry("Units", self.settings.units.name(), true),
            settings_entry("Theme", theme_name(self.settings.theme), true),
            settings_entry(
                "Thunder Flash",
                if self.settings.no_flash { "Off" } else { "On" },
                true,
            ),
            settings_entry("Icons", icon_mode_name(self.settings.icon_mode), true),
            settings_entry(
                "Inline Hints",
                if self.settings.inline_hints {
                    "Enabled"
                } else {
                    "Disabled"
                },
                true,
            ),
            settings_entry(
                "Command Bar",
                if self.settings.command_bar_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                true,
            ),
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

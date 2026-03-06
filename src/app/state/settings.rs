mod adjust;
mod options;

pub(crate) use adjust::adjust_setting_selection;
pub(super) use adjust::cycle;
pub(super) use options::HOURLY_VIEW_OPTIONS;
pub(crate) use options::SETTINGS_ORDER;
pub use options::SettingsSelection;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::domain::weather::HourlyViewMode;

    fn state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    #[test]
    fn settings_selection_navigation_is_bounded() {
        assert_eq!(SettingsSelection::Theme.prev(), SettingsSelection::Theme);
        assert_eq!(SettingsSelection::Close.next(), SettingsSelection::Close);
        assert_eq!(SettingsSelection::default(), SettingsSelection::Theme);
    }

    #[test]
    fn cycle_wraps_in_both_directions() {
        let values = [1, 2, 3];
        assert_eq!(adjust::cycle(&values, 1, 1), 2);
        assert_eq!(adjust::cycle(&values, 3, 1), 1);
        assert_eq!(adjust::cycle(&values, 1, -1), 3);
        assert_eq!(adjust::cycle::<u8>(&[], 7, 1), 7);
    }

    #[test]
    fn settings_entries_include_actions_and_editable_rows() {
        let state = state();
        let entries = state.settings_entries();
        assert_eq!(entries.len(), 12);
        assert!(entries[0].editable);
        assert_eq!(entries[0].label, "Theme");
        assert_eq!(entries[1].label, "Motion");
        assert_eq!(entries[4].label, "Hero Visual");
        assert_eq!(entries[8].label, "Units");
        assert_eq!(entries[10].label, "Action");
        assert!(!entries[10].editable);
        assert_eq!(entries[11].label, "Panel");
        assert!(!entries[11].editable);
    }

    #[test]
    fn settings_hint_changes_for_hero_visual_selection() {
        let mut state = state();
        state.settings_selected = SettingsSelection::Theme;
        assert!(state.settings_hint().contains("Theme applies"));

        state.settings_selected = SettingsSelection::HeroVisual;
        assert!(state.settings_hint().contains("Current panel right side"));

        state.settings_selected = SettingsSelection::Icons;
        assert!(state.settings_hint().contains("Nerd Font requires"));
    }

    #[test]
    fn adjust_setting_selection_handles_non_editable_and_flash() {
        let mut state = state();
        assert!(!adjust_setting_selection(
            &mut state,
            SettingsSelection::RefreshNow,
            1
        ));
        assert!(!adjust_setting_selection(
            &mut state,
            SettingsSelection::Close,
            1
        ));

        let before = state.settings.no_flash;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::Flash,
            1
        ));
        assert_ne!(state.settings.no_flash, before);
    }

    #[test]
    fn adjust_setting_selection_cycles_major_editable_fields() {
        let mut state = state();

        let units_before = state.settings.units;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::Units,
            1
        ));
        assert_ne!(state.settings.units, units_before);

        let theme_before = state.settings.theme;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::Theme,
            1
        ));
        assert_ne!(state.settings.theme, theme_before);

        let icon_before = state.settings.icon_mode;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::Icons,
            1
        ));
        assert_ne!(state.settings.icon_mode, icon_before);

        let inline_hints_before = state.settings.inline_hints;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::InlineHints,
            1
        ));
        assert_ne!(state.settings.inline_hints, inline_hints_before);
    }

    #[test]
    fn adjust_hourly_and_refresh_settings_update_runtime_values() {
        let mut state = state();
        state.hourly_view_mode = HourlyViewMode::Chart;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::HourlyView,
            1
        ));
        assert_eq!(state.settings.hourly_view, HourlyViewMode::Table);

        let refresh_before = state.settings.refresh_interval_secs;
        assert!(adjust_setting_selection(
            &mut state,
            SettingsSelection::RefreshInterval,
            1
        ));
        assert_ne!(state.settings.refresh_interval_secs, refresh_before);
    }
}

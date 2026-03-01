mod city_picker;
mod settings;

#[cfg(test)]
mod tests {
    use crate::app::state::{AppEvent, AppMode, AppState, LocationKey, RecentLocation};
    use crate::test_support;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tokio::sync::mpsc;

    use super::*;

    fn state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    #[test]
    fn test_handle_vertical_nav_movement_cases() {
        let cases = [
            (5, 10, KeyCode::Up, 4),
            (0, 10, KeyCode::Up, 10),
            (5, 10, KeyCode::Down, 6),
            (10, 10, KeyCode::Down, 0),
        ];

        for (start, max_index, key, expected) in cases {
            let mut selected = start;
            let handled = city_picker::handle_vertical_nav(&mut selected, max_index, key);
            assert!(handled);
            assert_eq!(selected, expected);
        }
    }

    #[test]
    fn test_handle_vertical_nav_other_key() {
        let mut selected = 5;
        let max_index = 10;
        let handled = city_picker::handle_vertical_nav(&mut selected, max_index, KeyCode::Left);
        assert!(!handled);
        assert_eq!(selected, 5);
    }

    #[test]
    fn ctrl_char_matches_case_insensitively_with_control_modifier() {
        let key = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL);
        assert!(settings::ctrl_char(key, 'c'));

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!settings::ctrl_char(key, 'x'));
    }

    #[tokio::test]
    async fn handle_help_key_closes_overlay_and_emits_shortcuts() {
        let mut state = state();
        state.help_open = true;
        let (tx, mut rx) = mpsc::channel(4);

        state
            .handle_help_key(
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
                &tx,
            )
            .await
            .expect("ctrl+l");
        assert!(matches!(rx.recv().await, Some(AppEvent::ForceRedraw)));

        state
            .handle_help_key(
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                &tx,
            )
            .await
            .expect("ctrl+c");
        assert!(matches!(rx.recv().await, Some(AppEvent::Quit)));

        state
            .handle_help_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &tx)
            .await
            .expect("esc");
        assert!(!state.help_open);
    }

    #[test]
    fn push_city_query_char_accepts_valid_and_ignores_control_chars() {
        let mut state = state();
        state.push_city_query_char(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE), 's');
        assert_eq!(state.city_query, "s");

        state.push_city_query_char(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            'x',
        );
        assert_eq!(state.city_query, "s");

        state.push_city_query_char(KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::NONE), '\n');
        assert_eq!(state.city_query, "s");
    }

    #[test]
    fn push_city_query_char_enforces_length_limit() {
        let mut state = state();
        // Fill up to 50
        for _ in 0..50 {
            state.push_city_query_char(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), 'a');
        }
        assert_eq!(state.city_query.len(), 50);

        // Try to add 51st
        state.push_city_query_char(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE), 'b');
        assert_eq!(state.city_query.len(), 50);
        assert!(!state.city_query.ends_with('b'));
    }

    #[test]
    fn clear_recent_locations_handles_empty_and_non_empty_states() {
        let mut state = state();
        state.clear_recent_locations();
        assert_eq!(
            state.city_status.as_deref(),
            Some("No recent locations to clear")
        );

        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(
                &test_support::stockholm_location(),
            ));
        state.clear_recent_locations();
        assert!(state.settings.recent_locations.is_empty());
        assert_eq!(
            state.city_status.as_deref(),
            Some("Cleared all recent locations")
        );
    }

    #[test]
    fn push_recent_location_deduplicates_and_respects_history_limit() {
        let mut state = state();
        let stockholm = test_support::stockholm_location();
        state.push_recent_location(&stockholm);
        state.push_recent_location(&stockholm);
        assert_eq!(state.settings.recent_locations.len(), 1);

        for idx in 0..20 {
            let mut loc = stockholm.clone();
            loc.name = format!("City {idx}");
            loc.latitude += f64::from(idx);
            loc.longitude += f64::from(idx);
            state.push_recent_location(&loc);
        }
        assert!(state.settings.recent_locations.len() <= 12);
    }

    #[test]
    fn city_picker_index_helpers_track_visible_rows() {
        let mut state = state();
        assert_eq!(state.visible_recent_count(), 0);
        assert_eq!(state.city_picker_action_index(), None);
        assert_eq!(state.city_picker_max_index(), 0);

        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(
                &test_support::stockholm_location(),
            ));
        assert_eq!(state.visible_recent_count(), 1);
        assert_eq!(state.city_picker_action_index(), Some(1));
        assert_eq!(state.city_picker_max_index(), 1);
    }

    #[test]
    fn submit_city_picker_can_clear_history_without_search() {
        let mut state = state();
        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(
                &test_support::stockholm_location(),
            ));
        state.city_history_selected = state.city_picker_action_index().expect("action index");
        let (tx, _rx) = mpsc::channel(2);
        let cli = test_support::state_test_cli();

        state.submit_city_picker(&tx, &cli);
        assert!(state.settings.recent_locations.is_empty());
        assert_eq!(
            state.city_status.as_deref(),
            Some("Cleared all recent locations")
        );
    }

    #[tokio::test]
    async fn submit_city_picker_keeps_picker_open_during_search() {
        let mut state = state();
        state.city_picker_open = true;
        state.city_query = "London".to_string();
        let (tx, _rx) = mpsc::channel(2);
        let cli = test_support::state_test_cli();

        state.submit_city_picker(&tx, &cli);
        assert!(state.city_picker_open);
        assert!(state.city_status.as_deref().unwrap().contains("Searching"));
    }

    #[tokio::test]
    async fn select_recent_city_keeps_picker_open_if_not_cached() {
        let mut state = state();
        state.city_picker_open = true;
        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(
                &test_support::stockholm_location(),
            ));
        let (tx, _rx) = mpsc::channel(2);

        // Ensure cache miss (cache is empty by default)
        state.select_recent_city_by_index(&tx, 0);

        assert!(state.city_picker_open);
        assert!(state.city_status.as_deref().unwrap().contains("Switching"));
    }

    #[tokio::test]
    async fn select_recent_city_by_index_switches_to_selected_location() {
        let mut state = state();
        let mut berlin = test_support::stockholm_location();
        berlin.name = "Berlin".to_string();
        berlin.latitude = 52.52;
        berlin.longitude = 13.405;
        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(&berlin));
        let (tx, _rx) = mpsc::channel(2);

        state.select_recent_city_by_index(&tx, 0);
        assert_eq!(
            state.selected_location.as_ref().map(|l| l.name.as_str()),
            Some("Berlin")
        );
    }

    #[test]
    fn handle_city_picker_nav_key_handles_up_down_esc_and_enter() {
        let mut state = state();
        state.city_picker_open = true;
        state
            .settings
            .recent_locations
            .push(RecentLocation::from_location(
                &test_support::stockholm_location(),
            ));
        let (tx, _rx) = mpsc::channel(2);
        let cli = test_support::state_test_cli();

        assert!(state.handle_city_picker_nav_key(KeyCode::Down, &tx, &cli));
        assert_eq!(state.city_history_selected, 1);
        assert!(state.handle_city_picker_nav_key(KeyCode::Up, &tx, &cli));
        assert_eq!(state.city_history_selected, 0);

        assert!(state.handle_city_picker_nav_key(KeyCode::Esc, &tx, &cli));
        assert!(!state.city_picker_open);
        assert!(state.city_status.is_none());
    }

    #[test]
    fn switch_to_location_uses_cache_without_fetch_when_fresh() {
        let mut state = state();
        let (tx, _rx) = mpsc::channel(2);
        let mut bundle = test_support::sample_bundle();
        bundle.fetched_at = chrono::Utc::now();
        let location = bundle.location.clone();
        let key: LocationKey = (&location).into();
        state.forecast_cache.put(key, bundle.clone());

        state.switch_to_location(&tx, location);
        assert_eq!(state.mode, AppMode::Ready);
        assert!(state.weather.is_some());
    }
}

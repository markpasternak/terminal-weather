use super::*;

impl AppState {
    pub(crate) async fn handle_settings_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if settings_close_key(code) {
            self.settings_open = false;
            return Ok(());
        }
        if self.handle_settings_nav_key(code) {
            return Ok(());
        }
        if matches!(code, KeyCode::Enter) {
            self.handle_settings_enter(tx, cli).await?;
        }
        Ok(())
    }

    fn handle_settings_nav_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Up => {
                self.settings_selected = self.settings_selected.prev();
                true
            }
            KeyCode::Down => {
                self.settings_selected = self.settings_selected.next();
                true
            }
            KeyCode::Left => {
                self.adjust_selected_setting(-1);
                true
            }
            KeyCode::Right => {
                self.adjust_selected_setting(1);
                true
            }
            _ => false,
        }
    }

    pub(crate) async fn handle_settings_enter(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match self.settings_selected {
            SettingsSelection::RefreshNow => self.start_fetch(tx, cli).await?,
            SettingsSelection::Close => self.settings_open = false,
            _ => self.adjust_selected_setting(1),
        }
        Ok(())
    }

    pub(crate) async fn handle_help_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<()> {
        if matches!(key.code, KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?')) {
            self.help_open = false;
            return Ok(());
        }
        if ctrl_char(key, 'c') {
            tx.send(AppEvent::Quit).await?;
            return Ok(());
        }
        if ctrl_char(key, 'l') {
            tx.send(AppEvent::ForceRedraw).await?;
        }
        Ok(())
    }

    pub(crate) fn handle_city_picker_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) {
        self.city_history_selected = self.city_history_selected.min(self.city_picker_max_index());
        if self.handle_city_picker_nav_key(key.code, tx, cli) {
            return;
        }
        match key.code {
            KeyCode::Backspace => {
                self.city_query.pop();
            }
            KeyCode::Delete => {
                self.clear_recent_locations();
            }
            KeyCode::Char(digit @ '1'..='9') => {
                self.select_recent_city_by_index(tx, (digit as usize) - ('1' as usize));
            }
            KeyCode::Char(ch) => {
                self.push_city_query_char(key, ch);
            }
            _ => {}
        }
    }

    pub(crate) fn handle_city_picker_nav_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> bool {
        let max_index = self.city_picker_max_index();
        if handle_vertical_nav(&mut self.city_history_selected, max_index, code) {
            return true;
        }
        match code {
            KeyCode::Esc => {
                self.city_picker_open = false;
                self.city_status = None;
                true
            }
            KeyCode::Enter => {
                self.submit_city_picker(tx, cli);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn select_recent_city_by_index(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        index: usize,
    ) {
        if let Some(saved) = self.settings.recent_locations.get(index).cloned() {
            self.city_picker_open = false;
            self.switch_to_location(tx, saved.to_location());
        }
    }

    pub(crate) fn submit_city_picker(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) {
        let query = self.city_query.trim().to_string();
        if !query.is_empty() {
            self.city_picker_open = false;
            self.city_status = Some(format!("Searching {query}..."));
            self.start_city_search(tx, query, cli.country_code.clone());
            return;
        }
        if Some(self.city_history_selected) == self.city_picker_action_index() {
            self.clear_recent_locations();
            return;
        }
        self.select_recent_city_by_index(tx, self.city_history_selected);
    }

    pub(crate) fn push_city_query_char(&mut self, key: KeyEvent, ch: char) {
        if !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
            && is_city_char(ch)
        {
            self.city_query.push(ch);
        }
    }

    pub(crate) fn adjust_selected_setting(&mut self, direction: i8) {
        let changed = adjust_setting_selection(self, self.settings_selected, direction);

        if changed {
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    pub(crate) fn apply_runtime_settings(&mut self) {
        self.units = self.settings.units;
        self.hourly_view_mode = self.settings.hourly_view;
        self.animate_ui = matches!(self.settings.motion, MotionSetting::Full);
        self.refresh_interval_secs_runtime
            .store(self.settings.refresh_interval_secs, Ordering::Relaxed);
        self.particles.set_options(
            matches!(self.settings.motion, MotionSetting::Off),
            matches!(self.settings.motion, MotionSetting::Reduced),
            self.settings.no_flash,
        );
    }

    pub(crate) fn persist_settings(&mut self) {
        if self.demo_mode {
            return;
        }
        if let Some(path) = &self.settings_path
            && let Err(err) = save_runtime_settings(path, &self.settings)
        {
            self.last_error = Some(format!("Failed to save settings: {err}"));
        }
    }

    pub(crate) fn push_recent_location(&mut self, location: &Location) {
        let entry = RecentLocation::from_location(location);
        self.settings
            .recent_locations
            .retain(|existing| !existing.same_place(&entry));
        self.settings.recent_locations.insert(0, entry);
        self.settings.recent_locations.truncate(HISTORY_MAX);
        self.city_history_selected = self
            .city_history_selected
            .min(self.settings.recent_locations.len().saturating_sub(1));
    }

    pub(crate) fn clear_recent_locations(&mut self) {
        if self.settings.recent_locations.is_empty() {
            self.city_status = Some("No recent locations to clear".to_string());
            self.city_history_selected = 0;
            return;
        }
        self.settings.recent_locations.clear();
        self.city_history_selected = 0;
        self.city_status = Some("Cleared all recent locations".to_string());
        self.persist_settings();
    }

    pub(crate) fn visible_recent_count(&self) -> usize {
        self.settings
            .recent_locations
            .len()
            .min(CITY_PICKER_VISIBLE_MAX)
    }

    pub(crate) fn city_picker_action_index(&self) -> Option<usize> {
        let visible = self.visible_recent_count();
        if visible > 0 { Some(visible) } else { None }
    }

    pub(crate) fn city_picker_max_index(&self) -> usize {
        self.city_picker_action_index().unwrap_or(0)
    }

    pub(crate) fn switch_to_location(&mut self, tx: &mpsc::Sender<AppEvent>, location: Location) {
        self.selected_location = Some(location.clone());
        self.pending_locations.clear();

        let key: LocationKey = (&location).into();
        if let Some(bundle) = self.forecast_cache.get(&key).cloned() {
            self.handle_fetch_succeeded(bundle.clone());
            if (chrono::Utc::now() - bundle.fetched_at) > chrono::Duration::minutes(10) {
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    let _ = tx2.send(AppEvent::FetchStarted).await;
                });
                self.fetch_forecast(tx, location);
            }
            return;
        }

        self.mode = AppMode::Loading;
        self.city_status = Some(format!("Switching to {}", location.display_name()));
        self.fetch_forecast(tx, location);
    }

    pub(crate) fn start_city_search(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        city: String,
        country_code: Option<String>,
    ) {
        self.pending_locations.clear();
        self.mode = AppMode::Loading;
        self.fetch_in_flight = true;
        self.loading_message = format!("Searching {city}...");
        self.refresh_meta.last_attempt = Some(chrono::Utc::now());

        let geocoder = GeocodeClient::new();
        let tx2 = tx.clone();
        tokio::spawn(async move {
            match geocoder.resolve(city, country_code).await {
                Ok(resolution) => {
                    let _ = tx2.send(AppEvent::GeocodeResolved(resolution)).await;
                }
                Err(err) => {
                    let _ = tx2.send(AppEvent::FetchFailed(err.to_string())).await;
                }
            }
        });
    }
}

fn ctrl_char(key: KeyEvent, target: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

fn handle_vertical_nav(selected: &mut usize, max_index: usize, code: KeyCode) -> bool {
    match code {
        KeyCode::Up => {
            *selected = selected.saturating_sub(1);
            true
        }
        KeyCode::Down => {
            *selected = (*selected + 1).min(max_index);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    #[test]
    fn test_handle_vertical_nav_movement_cases() {
        let cases = [
            (5, 10, KeyCode::Up, 4),
            (0, 10, KeyCode::Up, 0),
            (5, 10, KeyCode::Down, 6),
            (10, 10, KeyCode::Down, 10),
        ];

        for (start, max_index, key, expected) in cases {
            let mut selected = start;
            let handled = handle_vertical_nav(&mut selected, max_index, key);
            assert!(handled);
            assert_eq!(selected, expected);
        }
    }

    #[test]
    fn test_handle_vertical_nav_other_key() {
        let mut selected = 5;
        let max_index = 10;
        let handled = handle_vertical_nav(&mut selected, max_index, KeyCode::Left);
        assert!(!handled);
        assert_eq!(selected, 5);
    }

    #[test]
    fn ctrl_char_matches_case_insensitively_with_control_modifier() {
        let key = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL);
        assert!(ctrl_char(key, 'c'));

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!ctrl_char(key, 'x'));
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
                &crate::test_support::stockholm_location(),
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
        let stockholm = crate::test_support::stockholm_location();
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
        assert!(state.settings.recent_locations.len() <= HISTORY_MAX);
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
                &crate::test_support::stockholm_location(),
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
                &crate::test_support::stockholm_location(),
            ));
        state.city_history_selected = state.city_picker_action_index().expect("action index");
        let (tx, _rx) = mpsc::channel(2);
        let cli = crate::test_support::state_test_cli();

        state.submit_city_picker(&tx, &cli);
        assert!(state.settings.recent_locations.is_empty());
        assert_eq!(
            state.city_status.as_deref(),
            Some("Cleared all recent locations")
        );
    }

    #[tokio::test]
    async fn select_recent_city_by_index_switches_to_selected_location() {
        let mut state = state();
        let mut berlin = crate::test_support::stockholm_location();
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
                &crate::test_support::stockholm_location(),
            ));
        let (tx, _rx) = mpsc::channel(2);
        let cli = crate::test_support::state_test_cli();

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
        let mut bundle = crate::test_support::sample_bundle();
        bundle.fetched_at = chrono::Utc::now();
        let location = bundle.location.clone();
        let key: LocationKey = (&location).into();
        state.forecast_cache.put(key, bundle.clone());

        state.switch_to_location(&tx, location);
        assert_eq!(state.mode, AppMode::Ready);
        assert!(state.weather.is_some());
    }
}

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
                self.settings_selected = self.settings_selected.saturating_sub(1);
                true
            }
            KeyCode::Down => {
                self.settings_selected = (self.settings_selected + 1).min(SETTINGS_COUNT - 1);
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
        if self.settings_selected == SETTINGS_REFRESH_NOW {
            self.start_fetch(tx, cli).await?;
        } else if self.settings_selected == SETTINGS_CLOSE {
            self.settings_open = false;
        } else {
            self.adjust_selected_setting(1);
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
        match code {
            KeyCode::Esc => {
                self.city_picker_open = false;
                self.city_status = None;
                true
            }
            KeyCode::Up => {
                self.city_history_selected = self.city_history_selected.saturating_sub(1);
                true
            }
            KeyCode::Down => {
                self.city_history_selected =
                    (self.city_history_selected + 1).min(self.city_picker_max_index());
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
        let changed = SETTINGS_ADJUSTERS
            .get(self.settings_selected)
            .is_some_and(|adjust| adjust(self, direction));
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
                Self::fetch_forecast(tx, location);
            }
            return;
        }

        self.mode = AppMode::Loading;
        self.city_status = Some(format!("Switching to {}", location.display_name()));
        Self::fetch_forecast(tx, location);
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

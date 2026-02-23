use crate::app::state::input::is_city_char;
use crate::app::state::{AppEvent, AppMode, AppState, LocationKey, RecentLocation};
use crate::cli::Cli;
use crate::data::geocode::GeocodeClient;
use crate::domain::weather::Location;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

const HISTORY_MAX: usize = 12;
const CITY_PICKER_VISIBLE_MAX: usize = 9;

pub(crate) fn handle_vertical_nav(selected: &mut usize, max_index: usize, code: KeyCode) -> bool {
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

impl AppState {
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
            && self.city_query.chars().count() < 50
        {
            self.city_query.push(ch);
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

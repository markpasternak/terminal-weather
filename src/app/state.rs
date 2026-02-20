use std::{io::IsTerminal, path::PathBuf, time::Instant};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tokio::sync::mpsc;

use crate::{
    app::{
        events::{
            AppEvent, DemoAction, schedule_retry, start_demo_task, start_frame_task,
            start_refresh_task,
        },
        settings::{
            MotionSetting, RecentLocation, RuntimeSettings, clear_runtime_settings,
            hourly_view_from_cli, load_runtime_settings, save_runtime_settings,
        },
    },
    cli::{Cli, ColorArg, HeroVisualArg, IconMode, ThemeArg},
    data::{forecast::ForecastClient, geocode::GeocodeClient},
    domain::weather::{
        ForecastBundle, GeocodeResolution, HourlyViewMode, Location, RefreshMetadata, Units,
        evaluate_freshness,
    },
    resilience::backoff::Backoff,
    ui::layout::visible_hour_count,
    ui::particles::ParticleEngine,
};

const SETTINGS_UNITS: usize = 0;
const SETTINGS_THEME: usize = 1;
const SETTINGS_MOTION: usize = 2;
const SETTINGS_FLASH: usize = 3;
const SETTINGS_ICONS: usize = 4;
const SETTINGS_HOURLY_VIEW: usize = 5;
const SETTINGS_HERO_VISUAL: usize = 6;
const SETTINGS_REFRESH_INTERVAL: usize = 7;
const SETTINGS_REFRESH_NOW: usize = 8;
const SETTINGS_CLOSE: usize = 9;
const SETTINGS_COUNT: usize = 10;

const REFRESH_OPTIONS: [u64; 4] = [300, 600, 900, 1800];
const HISTORY_MAX: usize = 12;
const CITY_PICKER_VISIBLE_MAX: usize = 9;
const HOURLY_VIEW_OPTIONS: [HourlyViewMode; 3] = [
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    SelectingLocation,
    Ready,
    Error,
    Quit,
}

#[derive(Debug, Clone)]
pub struct SettingsEntry {
    pub label: &'static str,
    pub value: String,
    pub editable: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub running: bool,
    pub loading_message: String,
    pub last_error: Option<String>,
    pub selected_location: Option<Location>,
    pub pending_locations: Vec<Location>,
    pub weather: Option<ForecastBundle>,
    pub refresh_meta: RefreshMetadata,
    pub units: Units,
    pub hourly_offset: usize,
    pub hourly_cursor: usize,
    pub particles: ParticleEngine,
    pub backoff: Backoff,
    pub fetch_in_flight: bool,
    pub last_frame_at: Instant,
    pub frame_tick: u64,
    pub animate_ui: bool,
    pub viewport_width: u16,
    pub demo_mode: bool,
    pub settings: RuntimeSettings,
    pub settings_open: bool,
    pub help_open: bool,
    pub settings_selected: usize,
    pub city_picker_open: bool,
    pub city_query: String,
    pub city_history_selected: usize,
    pub city_status: Option<String>,
    pub color_mode: ColorArg,
    pub hourly_view_mode: HourlyViewMode,
    settings_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(cli: &Cli) -> Self {
        let (mut settings, settings_path) =
            load_runtime_settings(cli, std::io::stdout().is_terminal());
        if cli.demo {
            if let Some(path) = settings_path.as_deref() {
                let _ = clear_runtime_settings(path);
            }
            settings = RuntimeSettings::default();
        }
        let disabled = matches!(settings.motion, MotionSetting::Off);
        let reduced = matches!(settings.motion, MotionSetting::Reduced);
        let runtime_hourly_view = cli
            .hourly_view
            .map_or(settings.hourly_view, hourly_view_from_cli);

        Self {
            mode: AppMode::Loading,
            running: true,
            loading_message: "Initializing...".to_string(),
            last_error: None,
            selected_location: None,
            pending_locations: Vec::new(),
            weather: None,
            refresh_meta: RefreshMetadata::default(),
            units: settings.units,
            hourly_offset: 0,
            hourly_cursor: 0,
            particles: ParticleEngine::new(disabled, reduced, settings.no_flash),
            backoff: Backoff::new(10, 300),
            fetch_in_flight: false,
            last_frame_at: Instant::now(),
            frame_tick: 0,
            animate_ui: matches!(settings.motion, MotionSetting::Full),
            viewport_width: 80,
            demo_mode: cli.demo,
            settings,
            settings_open: false,
            help_open: false,
            settings_selected: 0,
            city_picker_open: false,
            city_query: String::new(),
            city_history_selected: 0,
            city_status: None,
            color_mode: cli.effective_color_mode(),
            hourly_view_mode: runtime_hourly_view,
            settings_path,
        }
    }

    #[must_use]
    pub fn settings_entries(&self) -> Vec<SettingsEntry> {
        vec![
            SettingsEntry {
                label: "Units",
                value: match self.settings.units {
                    Units::Celsius => "Celsius".to_string(),
                    Units::Fahrenheit => "Fahrenheit".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Theme",
                value: match self.settings.theme {
                    ThemeArg::Auto => "Auto".to_string(),
                    ThemeArg::Aurora => "Aurora".to_string(),
                    ThemeArg::MidnightCyan => "Midnight Cyan".to_string(),
                    ThemeArg::Aubergine => "Aubergine".to_string(),
                    ThemeArg::Hoth => "Hoth".to_string(),
                    ThemeArg::Monument => "Monument".to_string(),
                    ThemeArg::Nord => "Nord".to_string(),
                    ThemeArg::CatppuccinMocha => "Catppuccin Mocha".to_string(),
                    ThemeArg::Mono => "Mono".to_string(),
                    ThemeArg::HighContrast => "High contrast".to_string(),
                    ThemeArg::Dracula => "Dracula".to_string(),
                    ThemeArg::GruvboxMaterialDark => "Gruvbox Material".to_string(),
                    ThemeArg::KanagawaWave => "Kanagawa Wave".to_string(),
                    ThemeArg::AyuMirage => "Ayu Mirage".to_string(),
                    ThemeArg::AyuLight => "Ayu Light".to_string(),
                    ThemeArg::PoimandresStorm => "Poimandres Storm".to_string(),
                    ThemeArg::SelenizedDark => "Selenized Dark".to_string(),
                    ThemeArg::NoClownFiesta => "No Clown Fiesta".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Motion",
                value: match self.settings.motion {
                    MotionSetting::Full => "Full".to_string(),
                    MotionSetting::Reduced => "Reduced".to_string(),
                    MotionSetting::Off => "Off".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Thunder Flash",
                value: if self.settings.no_flash {
                    "Off".to_string()
                } else {
                    "On".to_string()
                },
                editable: true,
            },
            SettingsEntry {
                label: "Icons",
                value: match self.settings.icon_mode {
                    IconMode::Unicode => "Unicode".to_string(),
                    IconMode::Ascii => "ASCII".to_string(),
                    IconMode::Emoji => "Emoji".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Hourly View",
                value: match self.hourly_view_mode {
                    HourlyViewMode::Table => "Table".to_string(),
                    HourlyViewMode::Hybrid => "Hybrid".to_string(),
                    HourlyViewMode::Chart => "Chart".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Hero Visual",
                value: match self.settings.hero_visual {
                    HeroVisualArg::AtmosCanvas => "Atmos Canvas".to_string(),
                    HeroVisualArg::GaugeCluster => "Gauge Cluster".to_string(),
                    HeroVisualArg::SkyObservatory => "Sky Observatory".to_string(),
                },
                editable: true,
            },
            SettingsEntry {
                label: "Auto Refresh",
                value: format!(
                    "{} min (next launch)",
                    self.settings.refresh_interval_secs / 60
                ),
                editable: true,
            },
            SettingsEntry {
                label: "Action",
                value: "Refresh now".to_string(),
                editable: false,
            },
            SettingsEntry {
                label: "Panel",
                value: "Close".to_string(),
                editable: false,
            },
        ]
    }

    #[must_use]
    pub fn settings_hint(&self) -> String {
        match self.settings_selected {
            SETTINGS_HERO_VISUAL => match self.settings.hero_visual {
                HeroVisualArg::AtmosCanvas => {
                    "Current panel right side: data-driven terrain + condition sky overlays"
                        .to_string()
                }
                HeroVisualArg::GaugeCluster => {
                    "Current panel right side: live instrument panel (temp, humidity, wind, pressure, UV)"
                        .to_string()
                }
                HeroVisualArg::SkyObservatory => {
                    "Current panel right side: sun/moon arc with weather strip and precipitation lane"
                        .to_string()
                }
            },
            SETTINGS_THEME => {
                "Theme applies to all panels: Current, Hourly, 7-Day, popups, and status"
                    .to_string()
            }
            SETTINGS_MOTION => {
                "Motion controls the moving effects: weather particles + animated hero scene (Full/Reduced/Off)"
                    .to_string()
            }
            SETTINGS_ICONS => {
                "Icon mode affects weather symbols in Hourly and 7-Day panels".to_string()
            }
            SETTINGS_HOURLY_VIEW => {
                "Hourly View controls the Hourly panel: Table, Hybrid cards+charts, or Chart"
                    .to_string()
            }
            SETTINGS_REFRESH_INTERVAL => {
                "Auto-refresh cadence persists and applies on next launch".to_string()
            }
            _ => "Use left/right or Enter to change the selected setting".to_string(),
        }
    }

    pub async fn handle_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if event_is_async(&event) {
            self.handle_async_event(event, tx, cli).await?;
        } else {
            self.handle_sync_event(event, tx)?;
        }

        Ok(())
    }

    async fn handle_async_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            AppEvent::Bootstrap => self.handle_bootstrap(tx, cli).await?,
            AppEvent::TickRefresh => self.handle_tick_refresh(tx, cli).await?,
            AppEvent::Input(event) => self.handle_input(event, tx, cli).await?,
            AppEvent::Demo(action) => self.handle_demo_action(action, tx).await?,
            _ => {}
        }
        Ok(())
    }

    fn handle_sync_event(&mut self, event: AppEvent, tx: &mpsc::Sender<AppEvent>) -> Result<()> {
        match event {
            AppEvent::TickFrame => self.handle_tick_frame(),
            AppEvent::FetchStarted => self.handle_fetch_started(),
            AppEvent::GeocodeResolved(resolution) => {
                self.handle_geocode_resolved(tx, resolution)?;
            }
            AppEvent::FetchSucceeded(bundle) => self.handle_fetch_succeeded(bundle),
            AppEvent::FetchFailed(err) => self.handle_fetch_failed(tx, err),
            AppEvent::Quit => self.mode = AppMode::Quit,
            _ => {}
        }
        Ok(())
    }

    async fn handle_bootstrap(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        cli.validate()?;
        let frame_fps = match self.settings.motion {
            MotionSetting::Full => cli.fps,
            MotionSetting::Reduced => cli.fps.min(20),
            MotionSetting::Off => 15,
        };
        start_frame_task(tx.clone(), frame_fps);
        start_refresh_task(tx.clone(), self.settings.refresh_interval_secs);
        if cli.demo {
            start_demo_task(tx.clone());
        }
        self.start_fetch(tx, cli).await
    }

    fn handle_tick_frame(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;
        self.frame_tick = self.frame_tick.saturating_add(1);

        self.particles.update(
            self.weather
                .as_ref()
                .map(ForecastBundle::current_weather_code),
            self.weather.as_ref().map(|w| w.current.wind_speed_10m),
            self.weather.as_ref().map(|w| w.current.wind_direction_10m),
            delta,
        );
        self.refresh_meta.state = evaluate_freshness(
            self.refresh_meta.last_success,
            self.refresh_meta.consecutive_failures,
        );
    }

    async fn handle_tick_refresh(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        if matches!(
            self.mode,
            AppMode::Ready | AppMode::Error | AppMode::Loading
        ) {
            self.start_fetch(tx, cli).await?;
        }
        Ok(())
    }

    fn handle_fetch_started(&mut self) {
        self.fetch_in_flight = true;
        self.loading_message = "Fetching weather...".to_string();
        if self.weather.is_none() {
            self.mode = AppMode::Loading;
        }
        self.refresh_meta.last_attempt = Some(chrono::Utc::now());
    }

    fn handle_geocode_resolved(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        resolution: GeocodeResolution,
    ) -> Result<()> {
        match resolution {
            GeocodeResolution::Selected(location) => {
                self.selected_location = Some(location.clone());
                self.pending_locations.clear();
                self.fetch_forecast(tx, location);
            }
            GeocodeResolution::NeedsDisambiguation(locations) => {
                self.pending_locations = locations;
                self.fetch_in_flight = false;
                self.mode = AppMode::SelectingLocation;
                self.loading_message = "Choose a location (1-5)".to_string();
                self.city_picker_open = false;
            }
            GeocodeResolution::NotFound(city) => {
                self.fetch_in_flight = false;
                self.mode = AppMode::Error;
                self.last_error = Some(format!("No geocoding result for {city}"));
            }
        }
        Ok(())
    }

    fn handle_fetch_succeeded(&mut self, bundle: ForecastBundle) {
        let location = bundle.location.clone();
        self.fetch_in_flight = false;
        self.weather = Some(bundle);
        self.mode = AppMode::Ready;
        self.last_error = None;
        self.refresh_meta.mark_success();
        self.backoff.reset();
        self.hourly_offset = 0;
        self.hourly_cursor = 0;
        self.push_recent_location(&location);
        self.persist_settings();
        self.city_status = None;
    }

    fn handle_fetch_failed(&mut self, tx: &mpsc::Sender<AppEvent>, err: String) {
        self.fetch_in_flight = false;
        self.last_error = Some(err);
        self.mode = AppMode::Error;
        self.city_status = Some("Search failed; keeping last successful weather".to_string());
        self.refresh_meta.mark_failure();
        self.refresh_meta.state = evaluate_freshness(
            self.refresh_meta.last_success,
            self.refresh_meta.consecutive_failures,
        );
        let delay = self.backoff.next_delay();
        schedule_retry(tx.clone(), delay);
    }

    async fn handle_input(
        &mut self,
        event: Event,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key_press(key, tx, cli).await?;
            }
            Event::Resize(width, _) => {
                self.viewport_width = width;
                self.particles.reset();
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if self.handle_control_shortcuts(key, tx).await? {
            return Ok(());
        }
        if self.handle_modal_key_press(key, tx, cli).await? {
            return Ok(());
        }
        self.handle_main_key_press(key, tx, cli).await
    }

    async fn handle_control_shortcuts(
        &self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<bool> {
        if matches!(key.code, KeyCode::Char('c' | 'C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            tx.send(AppEvent::Quit).await?;
            return Ok(true);
        }
        if matches!(key.code, KeyCode::Char('l' | 'L'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            tx.send(AppEvent::ForceRedraw).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn handle_modal_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        if self.settings_open {
            self.handle_settings_key(key.code, tx, cli).await?;
            return Ok(true);
        }
        if self.city_picker_open {
            self.handle_city_picker_key(key, tx, cli).await?;
            return Ok(true);
        }
        if self.help_open {
            self.handle_help_key(key, tx).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn handle_main_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.open_help_overlay();
            }
            KeyCode::Esc => {
                tx.send(AppEvent::Quit).await?;
            }
            KeyCode::Left => {
                self.move_hourly_cursor_left();
            }
            KeyCode::Right => {
                self.move_hourly_cursor_right();
            }
            KeyCode::Char(digit @ '1'..='5') if self.mode == AppMode::SelectingLocation => {
                self.select_pending_location(tx, digit)?;
            }
            KeyCode::Char(_) => {
                self.handle_char_command(key, tx, cli).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_char_command(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        if command_char_matches(key, 'q') {
            tx.send(AppEvent::Quit).await?;
            return Ok(true);
        }
        if command_char_matches(key, 's') {
            if self.mode != AppMode::SelectingLocation {
                self.open_settings_panel();
            }
            return Ok(true);
        }
        if command_char_matches(key, 'l') {
            if self.mode != AppMode::SelectingLocation {
                self.open_city_picker();
            }
            return Ok(true);
        }
        if command_char_matches(key, 'r') {
            self.start_fetch(tx, cli).await?;
            return Ok(true);
        }
        if command_char_matches(key, 'f') {
            self.set_units(Units::Fahrenheit);
            return Ok(true);
        }
        if command_char_matches(key, 'c') {
            self.set_units(Units::Celsius);
            return Ok(true);
        }
        if command_char_matches(key, 'v') {
            self.settings.hourly_view = cycle(&HOURLY_VIEW_OPTIONS, self.hourly_view_mode, 1);
            self.apply_runtime_settings();
            self.persist_settings();
            return Ok(true);
        }
        Ok(false)
    }

    fn select_pending_location(&mut self, tx: &mpsc::Sender<AppEvent>, digit: char) -> Result<()> {
        let idx = (digit as usize) - ('1' as usize);
        if let Some(selected) = self.pending_locations.get(idx).cloned() {
            self.selected_location = Some(selected.clone());
            self.pending_locations.clear();
            self.mode = AppMode::Loading;
            self.fetch_forecast(tx, selected);
        }
        Ok(())
    }

    fn open_help_overlay(&mut self) {
        self.help_open = true;
        self.settings_open = false;
        self.city_picker_open = false;
    }

    fn open_settings_panel(&mut self) {
        self.city_picker_open = false;
        self.help_open = false;
        self.settings_open = true;
        self.settings_selected = 0;
    }

    fn open_city_picker(&mut self) {
        self.settings_open = false;
        self.help_open = false;
        self.city_picker_open = true;
        self.city_query.clear();
        self.city_history_selected = 0;
        self.city_status = Some("Type a city and press Enter, or pick from history".to_string());
    }

    fn set_units(&mut self, units: Units) {
        if self.settings.units != units {
            self.settings.units = units;
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    fn move_hourly_cursor_left(&mut self) {
        if self.hourly_cursor > 0 {
            self.hourly_cursor -= 1;
            if self.hourly_cursor < self.hourly_offset {
                self.hourly_offset = self.hourly_cursor;
            }
        }
    }

    fn move_hourly_cursor_right(&mut self) {
        if let Some(bundle) = &self.weather {
            let max = bundle.hourly.len().saturating_sub(1);
            if self.hourly_cursor < max {
                self.hourly_cursor += 1;
                let visible = visible_hour_count(self.viewport_width);
                if self.hourly_cursor >= self.hourly_offset + visible {
                    self.hourly_offset =
                        self.hourly_cursor.saturating_sub(visible.saturating_sub(1));
                }
            }
        }
    }

    async fn handle_settings_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match code {
            KeyCode::Esc => {
                self.settings_open = false;
            }
            KeyCode::Char(_) if command_char_matches_keycode(code, 's') => {
                self.settings_open = false;
            }
            KeyCode::Char(_) if command_char_matches_keycode(code, 'q') => {
                self.settings_open = false;
            }
            KeyCode::Up => {
                self.settings_selected = self.settings_selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.settings_selected = (self.settings_selected + 1).min(SETTINGS_COUNT - 1);
            }
            KeyCode::Left => {
                self.adjust_selected_setting(-1);
            }
            KeyCode::Right => {
                self.adjust_selected_setting(1);
            }
            KeyCode::Enter => {
                if self.settings_selected == SETTINGS_REFRESH_NOW {
                    self.start_fetch(tx, cli).await?;
                } else if self.settings_selected == SETTINGS_CLOSE {
                    self.settings_open = false;
                } else {
                    self.adjust_selected_setting(1);
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_help_key(&mut self, key: KeyEvent, tx: &mpsc::Sender<AppEvent>) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') => {
                self.help_open = false;
            }
            KeyCode::Char('c' | 'C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                tx.send(AppEvent::Quit).await?;
            }
            KeyCode::Char('l' | 'L') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                tx.send(AppEvent::ForceRedraw).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_city_picker_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        self.city_history_selected = self.city_history_selected.min(self.city_picker_max_index());
        match key.code {
            KeyCode::Esc => {
                self.city_picker_open = false;
                self.city_status = None;
            }
            KeyCode::Up => {
                self.city_history_selected = self.city_history_selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.city_history_selected =
                    (self.city_history_selected + 1).min(self.city_picker_max_index());
            }
            KeyCode::Backspace => {
                self.city_query.pop();
            }
            KeyCode::Delete => {
                self.clear_recent_locations();
            }
            KeyCode::Char(digit @ '1'..='9') => {
                self.select_recent_city_by_index(tx, (digit as usize) - ('1' as usize));
            }
            KeyCode::Enter => {
                self.submit_city_picker(tx, cli).await?;
            }
            KeyCode::Char(ch) => {
                self.push_city_query_char(key, ch);
            }
            _ => {}
        }
        Ok(())
    }

    fn select_recent_city_by_index(&mut self, tx: &mpsc::Sender<AppEvent>, index: usize) {
        if let Some(saved) = self.settings.recent_locations.get(index).cloned() {
            self.city_picker_open = false;
            self.switch_to_location(tx, saved.to_location());
        }
    }

    async fn submit_city_picker(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        let query = self.city_query.trim().to_string();
        if !query.is_empty() {
            self.city_picker_open = false;
            self.city_status = Some(format!("Searching {query}..."));
            self.start_city_search(tx, query, cli.country_code.clone());
            return Ok(());
        }
        if Some(self.city_history_selected) == self.city_picker_action_index() {
            self.clear_recent_locations();
            return Ok(());
        }
        self.select_recent_city_by_index(tx, self.city_history_selected);
        Ok(())
    }

    fn push_city_query_char(&mut self, key: KeyEvent, ch: char) {
        if !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
            && is_city_char(ch)
        {
            self.city_query.push(ch);
        }
    }

    fn adjust_selected_setting(&mut self, direction: i8) {
        let mut changed = false;
        match self.settings_selected {
            SETTINGS_UNITS => {
                self.settings.units = cycle(
                    &[Units::Celsius, Units::Fahrenheit],
                    self.settings.units,
                    direction,
                );
                changed = true;
            }
            SETTINGS_THEME => {
                self.settings.theme = cycle(&THEME_OPTIONS, self.settings.theme, direction);
                changed = true;
            }
            SETTINGS_MOTION => {
                self.settings.motion = cycle(
                    &[
                        MotionSetting::Full,
                        MotionSetting::Reduced,
                        MotionSetting::Off,
                    ],
                    self.settings.motion,
                    direction,
                );
                changed = true;
            }
            SETTINGS_FLASH => {
                self.settings.no_flash = !self.settings.no_flash;
                changed = true;
            }
            SETTINGS_ICONS => {
                self.settings.icon_mode = cycle(
                    &[IconMode::Unicode, IconMode::Ascii, IconMode::Emoji],
                    self.settings.icon_mode,
                    direction,
                );
                changed = true;
            }
            SETTINGS_HOURLY_VIEW => {
                self.settings.hourly_view =
                    cycle(&HOURLY_VIEW_OPTIONS, self.hourly_view_mode, direction);
                changed = true;
            }
            SETTINGS_HERO_VISUAL => {
                self.settings.hero_visual = cycle(
                    &[
                        HeroVisualArg::AtmosCanvas,
                        HeroVisualArg::GaugeCluster,
                        HeroVisualArg::SkyObservatory,
                    ],
                    self.settings.hero_visual,
                    direction,
                );
                changed = true;
            }
            SETTINGS_REFRESH_INTERVAL => {
                self.settings.refresh_interval_secs = cycle(
                    &REFRESH_OPTIONS,
                    self.settings.refresh_interval_secs,
                    direction,
                );
                changed = true;
            }
            _ => {}
        }

        if changed {
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    fn apply_runtime_settings(&mut self) {
        self.units = self.settings.units;
        self.hourly_view_mode = self.settings.hourly_view;
        self.animate_ui = matches!(self.settings.motion, MotionSetting::Full);
        self.particles.set_options(
            matches!(self.settings.motion, MotionSetting::Off),
            matches!(self.settings.motion, MotionSetting::Reduced),
            self.settings.no_flash,
        );
    }

    fn persist_settings(&mut self) {
        if self.demo_mode {
            return;
        }
        if let Some(path) = &self.settings_path
            && let Err(err) = save_runtime_settings(path, &self.settings)
        {
            self.last_error = Some(format!("Failed to save settings: {err}"));
        }
    }

    fn push_recent_location(&mut self, location: &Location) {
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

    fn clear_recent_locations(&mut self) {
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

    fn visible_recent_count(&self) -> usize {
        self.settings
            .recent_locations
            .len()
            .min(CITY_PICKER_VISIBLE_MAX)
    }

    fn city_picker_action_index(&self) -> Option<usize> {
        let visible = self.visible_recent_count();
        if visible > 0 { Some(visible) } else { None }
    }

    fn city_picker_max_index(&self) -> usize {
        self.city_picker_action_index().unwrap_or(0)
    }

    fn switch_to_location(&mut self, tx: &mpsc::Sender<AppEvent>, location: Location) {
        self.selected_location = Some(location.clone());
        self.pending_locations.clear();
        self.mode = AppMode::Loading;
        self.city_status = Some(format!("Switching to {}", location.display_name()));
        self.fetch_forecast(tx, location);
    }

    fn start_city_search(
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

    async fn start_fetch(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        if self.fetch_in_flight || self.mode == AppMode::SelectingLocation {
            return Ok(());
        }

        tx.send(AppEvent::FetchStarted).await?;

        if let Some(location) = self.selected_location.clone() {
            self.fetch_forecast(tx, location);
            return Ok(());
        }

        if let (Some(lat), Some(lon)) = (cli.lat, cli.lon) {
            let location = Location::from_coords(lat, lon);
            tx.send(AppEvent::GeocodeResolved(GeocodeResolution::Selected(
                location,
            )))
            .await?;
            return Ok(());
        }

        if cli.city.is_none() && cli.lat.is_none() {
            self.loading_message = "Detecting location...".to_string();
            let tx2 = tx.clone();
            let country_code = cli.country_code.clone();
            tokio::spawn(async move {
                // Try IP-based geolocation first
                if let Some(location) = crate::data::geoip::detect_location().await {
                    let _ = tx2
                        .send(AppEvent::GeocodeResolved(GeocodeResolution::Selected(
                            location,
                        )))
                        .await;
                    return;
                }
                // Fall back to Stockholm
                let geocoder = GeocodeClient::new();
                match geocoder
                    .resolve("Stockholm".to_string(), country_code)
                    .await
                {
                    Ok(resolution) => {
                        let _ = tx2.send(AppEvent::GeocodeResolved(resolution)).await;
                    }
                    Err(err) => {
                        let _ = tx2.send(AppEvent::FetchFailed(err.to_string())).await;
                    }
                }
            });
        } else {
            let geocoder = GeocodeClient::new();
            let city = cli.default_city();
            let country_code = cli.country_code.clone();
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

        Ok(())
    }

    fn fetch_forecast(&self, tx: &mpsc::Sender<AppEvent>, location: Location) {
        let client = ForecastClient::new();
        let tx2 = tx.clone();
        tokio::spawn(async move {
            match client.fetch(location).await {
                Ok(data) => {
                    let _ = tx2.send(AppEvent::FetchSucceeded(data)).await;
                }
                Err(err) => {
                    let _ = tx2.send(AppEvent::FetchFailed(err.to_string())).await;
                }
            }
        });
    }

    async fn handle_demo_action(
        &mut self,
        action: DemoAction,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<()> {
        match action {
            DemoAction::OpenCityPicker(query) => {
                self.settings_open = false;
                self.city_picker_open = true;
                self.city_query.clone_from(&query);
                self.city_history_selected = 0;
                self.city_status = Some(format!("Demo: search for {query}"));
            }
            DemoAction::SwitchCity(location) => {
                self.settings_open = false;
                self.city_picker_open = true;
                self.city_status = Some(format!("Demo: selected {}", location.display_name()));
                self.city_query.clear();
                self.city_picker_open = false;
                self.switch_to_location(tx, location);
            }
            DemoAction::OpenSettings => {
                self.city_picker_open = false;
                self.settings_open = true;
                self.settings_selected = SETTINGS_HERO_VISUAL;
            }
            DemoAction::SetHeroVisual(visual) => {
                self.settings_open = true;
                self.settings_selected = SETTINGS_HERO_VISUAL;
                if self.settings.hero_visual != visual {
                    self.settings.hero_visual = visual;
                    self.apply_runtime_settings();
                    self.persist_settings();
                }
            }
            DemoAction::SetTheme(theme) => {
                self.settings_open = true;
                self.settings_selected = SETTINGS_THEME;
                if self.settings.theme != theme {
                    self.settings.theme = theme;
                    self.apply_runtime_settings();
                    self.persist_settings();
                }
            }
            DemoAction::CloseSettings => {
                self.settings_open = false;
            }
            DemoAction::Quit => {
                tx.send(AppEvent::Quit).await?;
            }
        }
        Ok(())
    }
}

fn cycle<T: Copy + Eq>(values: &[T], current: T, direction: i8) -> T {
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

fn event_is_async(event: &AppEvent) -> bool {
    matches!(
        event,
        AppEvent::Bootstrap | AppEvent::TickRefresh | AppEvent::Input(_) | AppEvent::Demo(_)
    )
}

fn is_city_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '\'' | '’' | ',' | '.')
}

fn command_char_matches(key: KeyEvent, target: char) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
        && matches!(key.code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

fn command_char_matches_keycode(code: KeyCode, target: char) -> bool {
    matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

#[cfg(test)]
mod tests {
    use super::is_city_char;

    #[test]
    fn city_input_accepts_unicode_letters() {
        assert!(is_city_char('å'));
        assert!(is_city_char('Å'));
        assert!(is_city_char('é'));
    }

    #[test]
    fn city_input_rejects_control_chars() {
        assert!(!is_city_char('\n'));
        assert!(!is_city_char('\t'));
    }
}

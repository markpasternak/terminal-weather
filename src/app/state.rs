#![allow(clippy::missing_errors_doc)]

use std::{
    io::IsTerminal,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lru::LruCache;
use tokio::sync::mpsc;

use crate::{
    app::{
        events::{
            AppEvent, DemoAction, schedule_retry, start_demo_task, start_frame_task,
            start_refresh_task,
        },
        settings::{
            RecentLocation, RuntimeSettings, clear_runtime_settings, hourly_view_from_cli,
            load_runtime_settings,
        },
    },
    cli::{Cli, ColorArg, HeroVisualArg, ThemeArg},
    data::{forecast::ForecastClient, geocode::GeocodeClient},
    domain::weather::{
        ForecastBundle, GeocodeResolution, HourlyViewMode, Location, RefreshMetadata, Units,
        evaluate_freshness,
    },
    resilience::backoff::Backoff,
    ui::animation::{AnimationClockState, MotionMode, SceneTransitionState, WeatherMotionProfile},
    ui::layout::visible_hour_count,
    ui::particles::ParticleEngine,
    update::UpdateStatus,
};

mod input;
mod methods_async;
mod methods_fetch;
mod methods_ui;
mod settings;

use input::command_char;
pub(crate) use input::initial_selected_location;
pub(crate) use settings::SETTINGS_ORDER;
pub use settings::SettingsSelection;
use settings::{HOURLY_VIEW_OPTIONS, cycle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    SelectingLocation,
    Ready,
    Error,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Hero,
    Hourly,
    Daily,
}

impl PanelFocus {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Hero => Self::Hourly,
            Self::Hourly => Self::Daily,
            Self::Daily => Self::Hero,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Hero => Self::Daily,
            Self::Hourly => Self::Hero,
            Self::Daily => Self::Hourly,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hero => "Hero",
            Self::Hourly => "Hourly",
            Self::Daily => "7-Day",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandBarState {
    pub open: bool,
    pub buffer: String,
    pub parse_error: Option<String>,
}

impl CommandBarState {
    pub fn open(&mut self) {
        self.open = true;
        self.parse_error = None;
        if !self.buffer.starts_with(':') {
            self.buffer.clear();
            self.buffer.push(':');
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.buffer.clear();
        self.parse_error = None;
    }
}

#[derive(Debug, Clone)]
pub struct SettingsEntry {
    pub label: &'static str,
    pub value: String,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSignature {
    pub location_name: Option<String>,
    pub weather_code: Option<u8>,
    pub is_day: bool,
    pub hero_visual: HeroVisualArg,
    pub motion_mode: MotionMode,
    pub freshness_state: crate::resilience::freshness::FreshnessState,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LocationKey {
    name: String,
    lat_bits: u64,
    lon_bits: u64,
    country: Option<String>,
    admin1: Option<String>,
}

impl From<&Location> for LocationKey {
    fn from(loc: &Location) -> Self {
        Self {
            name: loc.name.clone(),
            lat_bits: loc.latitude.to_bits(),
            lon_bits: loc.longitude.to_bits(),
            country: loc.country.clone(),
            admin1: loc.admin1.clone(),
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub running: bool,
    pub loading_message: String,
    pub last_error: Option<String>,
    pub selected_location: Option<Location>,
    pub pending_locations: Vec<Location>,
    pub weather: Option<ForecastBundle>,
    pub forecast_cache: LruCache<LocationKey, ForecastBundle>,
    pub refresh_meta: RefreshMetadata,
    pub units: Units,
    pub hourly_offset: usize,
    pub hourly_cursor: usize,
    pub particles: ParticleEngine,
    pub backoff: Backoff,
    pub fetch_in_flight: bool,
    pub last_frame_at: Instant,
    pub frame_tick: u64,
    pub animation_clock: AnimationClockState,
    pub motion_mode: MotionMode,
    pub active_transition: Option<SceneTransitionState>,
    pub last_render_signature: Option<RenderSignature>,
    pub weather_motion_profile: Option<WeatherMotionProfile>,
    pub animate_ui: bool,
    pub viewport_width: u16,
    pub demo_mode: bool,
    pub settings: RuntimeSettings,
    pub settings_open: bool,
    pub help_open: bool,
    pub settings_selected: SettingsSelection,
    pub city_picker_open: bool,
    pub city_query: String,
    pub city_history_selected: usize,
    pub city_status: Option<String>,
    pub color_mode: ColorArg,
    pub hourly_view_mode: HourlyViewMode,
    pub panel_focus: PanelFocus,
    pub update_status: UpdateStatus,
    pub command_bar: CommandBarState,
    pub refresh_interval_secs_runtime: Arc<AtomicU64>,
    forecast_url_override: Option<String>,
    air_quality_url_override: Option<String>,
    settings_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(cli: &Cli) -> Self {
        let (settings, settings_path) = load_or_reset_settings(cli);
        let runtime_hourly_view = cli
            .hourly_view
            .map_or(settings.hourly_view, hourly_view_from_cli);
        let selected_location = initial_selected_location(cli, &settings);
        let refresh_interval_secs_runtime =
            Arc::new(AtomicU64::new(settings.refresh_interval_secs));
        let mut state =
            Self::new_runtime_state(&settings, selected_location, refresh_interval_secs_runtime);
        state.apply_cli_runtime_defaults(cli, settings, settings_path, runtime_hourly_view);
        state
    }

    fn new_runtime_state(
        settings: &RuntimeSettings,
        selected_location: Option<Location>,
        refresh_interval_secs_runtime: Arc<AtomicU64>,
    ) -> Self {
        Self {
            mode: AppMode::Loading,
            running: true,
            loading_message: "Initializing...".to_string(),
            last_error: None,
            selected_location,
            pending_locations: Vec::new(),
            weather: None,
            forecast_cache: LruCache::new(NonZeroUsize::new(10).unwrap_or(NonZeroUsize::MIN)),
            refresh_meta: RefreshMetadata::default(),
            units: settings.units,
            hourly_offset: 0,
            hourly_cursor: 0,
            particles: ParticleEngine::new(settings.motion_mode, settings.no_flash),
            backoff: Backoff::new(10, 300),
            fetch_in_flight: false,
            last_frame_at: Instant::now(),
            frame_tick: 0,
            animation_clock: AnimationClockState::default(),
            motion_mode: settings.motion_mode,
            active_transition: None,
            last_render_signature: None,
            weather_motion_profile: None,
            animate_ui: settings.motion_mode.allows_animation(),
            viewport_width: 80,
            demo_mode: false,
            settings: settings.clone(),
            settings_open: false,
            help_open: false,
            settings_selected: SettingsSelection::default(),
            city_picker_open: false,
            city_query: String::new(),
            city_history_selected: 0,
            city_status: None,
            color_mode: ColorArg::Auto,
            hourly_view_mode: settings.hourly_view,
            panel_focus: PanelFocus::Hourly,
            update_status: UpdateStatus::Unknown,
            command_bar: CommandBarState::default(),
            refresh_interval_secs_runtime,
            forecast_url_override: None,
            air_quality_url_override: None,
            settings_path: None,
        }
    }

    fn apply_cli_runtime_defaults(
        &mut self,
        cli: &Cli,
        settings: RuntimeSettings,
        settings_path: Option<PathBuf>,
        runtime_hourly_view: HourlyViewMode,
    ) {
        self.demo_mode = cli.demo;
        self.settings = settings;
        self.settings_path = settings_path;
        self.color_mode = cli.effective_color_mode();
        self.hourly_view_mode = runtime_hourly_view;
        self.forecast_url_override = cli.forecast_url.clone().filter(|url| {
            let lower = url.trim().to_ascii_lowercase();
            lower.starts_with("https://")
                || lower.starts_with("http://127.0.0.1")
                || lower.starts_with("http://localhost")
        });
        self.air_quality_url_override = cli.air_quality_url.clone().filter(|url| {
            let lower = url.trim().to_ascii_lowercase();
            lower.starts_with("https://")
                || lower.starts_with("http://127.0.0.1")
                || lower.starts_with("http://localhost")
        });
    }

    #[must_use]
    pub fn render_signature(&self) -> RenderSignature {
        RenderSignature {
            location_name: self
                .selected_location
                .as_ref()
                .map(crate::domain::weather::Location::display_name),
            weather_code: self
                .weather
                .as_ref()
                .map(|bundle| bundle.current.weather_code),
            is_day: self
                .weather
                .as_ref()
                .is_some_and(|bundle| bundle.current.is_day),
            hero_visual: self.settings.hero_visual,
            motion_mode: self.motion_mode,
            freshness_state: self.refresh_meta.state,
        }
    }

    pub(crate) fn begin_transition(&mut self, transition: Option<SceneTransitionState>) {
        self.active_transition = transition;
    }

    pub(crate) fn sync_motion_profile(&mut self) {
        self.weather_motion_profile = self.weather.as_ref().map(|bundle| {
            WeatherMotionProfile::from_bundle(bundle, self.active_transition.is_some())
        });
    }

    #[must_use]
    pub fn transition_progress(&self) -> Option<f32> {
        self.active_transition
            .map(SceneTransitionState::eased_progress)
    }

    #[must_use]
    pub fn motion_seed(&self, lane: &str) -> u64 {
        crate::ui::animation::stable_hash(&(
            self.selected_location
                .as_ref()
                .map(crate::domain::weather::Location::display_name),
            self.weather
                .as_ref()
                .map(|bundle| bundle.current.weather_code),
            self.weather
                .as_ref()
                .is_some_and(|bundle| bundle.current.is_day),
            self.settings.hero_visual as u8,
            self.motion_mode,
            lane,
        ))
    }
}

fn load_or_reset_settings(cli: &Cli) -> (RuntimeSettings, Option<PathBuf>) {
    let enable_disk = !cfg!(test) && std::io::stdout().is_terminal();
    let (mut settings, settings_path) = load_runtime_settings(cli, enable_disk);
    if cli.demo {
        if let Some(path) = settings_path.as_deref() {
            let _ = clear_runtime_settings(path);
        }
        settings = RuntimeSettings::default();
    }
    (settings, settings_path)
}

fn event_is_async(event: &AppEvent) -> bool {
    matches!(
        event,
        AppEvent::Bootstrap | AppEvent::TickRefresh | AppEvent::Input(_) | AppEvent::Demo(_)
    )
}

#[cfg(test)]
mod tests;

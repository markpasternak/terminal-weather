#![allow(clippy::missing_errors_doc)]

use std::{
    io::IsTerminal,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
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

mod input;
mod methods_async;
mod methods_fetch;
mod methods_ui;
mod settings;

use input::{command_char, settings_close_key};
pub(crate) use input::{initial_selected_location, is_city_char};
pub use settings::SettingsSelection;
pub(crate) use settings::adjust_setting_selection;
use settings::{HOURLY_VIEW_OPTIONS, cycle};

const HISTORY_MAX: usize = 12;
const CITY_PICKER_VISIBLE_MAX: usize = 9;

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
    pub refresh_interval_secs_runtime: Arc<AtomicU64>,
    forecast_url_override: Option<String>,
    air_quality_url_override: Option<String>,
    settings_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(cli: &Cli) -> Self {
        let (settings, settings_path) = load_or_reset_settings(cli);
        let (disabled, reduced) = motion_flags(&settings);
        let runtime_hourly_view = cli
            .hourly_view
            .map_or(settings.hourly_view, hourly_view_from_cli);
        let selected_location = initial_selected_location(cli, &settings);
        let refresh_interval_secs_runtime =
            Arc::new(AtomicU64::new(settings.refresh_interval_secs));

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
            settings_selected: SettingsSelection::default(),
            city_picker_open: false,
            city_query: String::new(),
            city_history_selected: 0,
            city_status: None,
            color_mode: cli.effective_color_mode(),
            hourly_view_mode: runtime_hourly_view,
            refresh_interval_secs_runtime,
            forecast_url_override: cli.forecast_url.clone(),
            air_quality_url_override: cli.air_quality_url.clone(),
            settings_path,
        }
    }
}

fn load_or_reset_settings(cli: &Cli) -> (RuntimeSettings, Option<PathBuf>) {
    let (mut settings, settings_path) = load_runtime_settings(cli, std::io::stdout().is_terminal());
    if cli.demo {
        if let Some(path) = settings_path.as_deref() {
            let _ = clear_runtime_settings(path);
        }
        settings = RuntimeSettings::default();
    }
    (settings, settings_path)
}

fn motion_flags(settings: &RuntimeSettings) -> (bool, bool) {
    (
        matches!(settings.motion, MotionSetting::Off),
        matches!(settings.motion, MotionSetting::Reduced),
    )
}

fn event_is_async(event: &AppEvent) -> bool {
    matches!(
        event,
        AppEvent::Bootstrap | AppEvent::TickRefresh | AppEvent::Input(_) | AppEvent::Demo(_)
    )
}

#[cfg(test)]
mod tests;

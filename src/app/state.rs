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

mod methods_async;
mod methods_fetch;
mod methods_ui;

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
            Self::RefreshNow => Self::Close,
            Self::Close => Self::Close,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Units => Self::Units,
            Self::Theme => Self::Units,
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

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

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
            forecast_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
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
            settings_path,
        }
    }

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

pub(crate) fn adjust_units_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.units = cycle(
        &[Units::Celsius, Units::Fahrenheit],
        state.settings.units,
        direction,
    );
    true
}

pub(crate) fn adjust_theme_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.theme = cycle(&THEME_OPTIONS, state.settings.theme, direction);
    true
}

pub(crate) fn adjust_motion_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.motion = cycle(
        &[
            MotionSetting::Full,
            MotionSetting::Reduced,
            MotionSetting::Off,
        ],
        state.settings.motion,
        direction,
    );
    true
}

pub(crate) fn adjust_flash_setting(state: &mut AppState, _: i8) -> bool {
    state.settings.no_flash = !state.settings.no_flash;
    true
}

pub(crate) fn adjust_icon_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.icon_mode = cycle(
        &[IconMode::Unicode, IconMode::Ascii, IconMode::Emoji],
        state.settings.icon_mode,
        direction,
    );
    true
}

pub(crate) fn adjust_hourly_view_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.hourly_view = cycle(&HOURLY_VIEW_OPTIONS, state.hourly_view_mode, direction);
    true
}

pub(crate) fn adjust_hero_visual_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.hero_visual = cycle(
        &[
            HeroVisualArg::AtmosCanvas,
            HeroVisualArg::GaugeCluster,
            HeroVisualArg::SkyObservatory,
        ],
        state.settings.hero_visual,
        direction,
    );
    true
}

pub(crate) fn adjust_refresh_interval_setting(state: &mut AppState, direction: i8) -> bool {
    state.settings.refresh_interval_secs = cycle(
        &REFRESH_OPTIONS,
        state.settings.refresh_interval_secs,
        direction,
    );
    true
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

fn initial_selected_location(cli: &Cli, settings: &RuntimeSettings) -> Option<Location> {
    if cli.city.is_some() || cli.lat.is_some() || cli.lon.is_some() || cli.demo {
        return None;
    }
    settings
        .recent_locations
        .first()
        .map(RecentLocation::to_location)
}

fn is_city_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '\'' | '’' | ',' | '.')
}

fn command_char(key: KeyEvent) -> Option<char> {
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }
    if let KeyCode::Char(ch) = key.code {
        Some(ch.to_ascii_lowercase())
    } else {
        None
    }
}

fn command_char_matches_keycode(code: KeyCode, target: char) -> bool {
    matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

fn settings_close_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc)
        || command_char_matches_keycode(code, 's')
        || command_char_matches_keycode(code, 'q')
}

#[cfg(test)]
mod tests {
    use super::{AppState, initial_selected_location, is_city_char};
    use crate::{
        app::settings::{RecentLocation, RuntimeSettings},
        cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
    };
    use std::sync::atomic::Ordering;

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

    #[test]
    fn initial_selected_location_uses_recent_when_no_cli_location() {
        let cli = test_cli();
        let mut settings = RuntimeSettings::default();
        settings.recent_locations.push(RecentLocation {
            name: "Stockholm".to_string(),
            latitude: 59.3293,
            longitude: 18.0686,
            country: Some("Sweden".to_string()),
            admin1: Some("Stockholm".to_string()),
            timezone: Some("Europe/Stockholm".to_string()),
        });

        let selected = initial_selected_location(&cli, &settings).expect("selected location");
        assert_eq!(selected.name, "Stockholm");
    }

    #[test]
    fn initial_selected_location_respects_cli_city_override() {
        let mut cli = test_cli();
        cli.city = Some("Berlin".to_string());
        let mut settings = RuntimeSettings::default();
        settings.recent_locations.push(RecentLocation {
            name: "Stockholm".to_string(),
            latitude: 59.3293,
            longitude: 18.0686,
            country: Some("Sweden".to_string()),
            admin1: Some("Stockholm".to_string()),
            timezone: Some("Europe/Stockholm".to_string()),
        });

        assert!(initial_selected_location(&cli, &settings).is_none());
    }

    #[test]
    fn settings_hint_explains_hero_visual() {
        let mut state = AppState::new(&test_cli());
        state.settings_selected = super::SettingsSelection::HeroVisual;
        assert!(state.settings_hint().contains("Current panel right side"));
    }

    #[test]
    fn apply_runtime_settings_updates_refresh_interval_runtime() {
        let mut state = AppState::new(&test_cli());
        state.settings.refresh_interval_secs = 300;
        state.apply_runtime_settings();
        assert_eq!(
            state.refresh_interval_secs_runtime.load(Ordering::Relaxed),
            300
        );
    }

    fn test_cli() -> Cli {
        Cli {
            city: None,
            units: UnitsArg::Celsius,
            fps: 30,
            no_animation: true,
            reduced_motion: false,
            no_flash: true,
            ascii_icons: false,
            emoji_icons: false,
            color: ColorArg::Auto,
            no_color: false,
            hourly_view: None,
            theme: ThemeArg::Auto,
            hero_visual: HeroVisualArg::AtmosCanvas,
            country_code: None,
            lat: None,
            lon: None,
            refresh_interval: 600,
            demo: false,
            one_shot: false,
        }
    }
}

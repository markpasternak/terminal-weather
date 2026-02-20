#![allow(clippy::cast_precision_loss)]

use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use rand::Rng;
use tokio::time::{interval, sleep};

use crate::{
    cli::{HeroVisualArg, ThemeArg},
    domain::weather::{ForecastBundle, GeocodeResolution, Location},
};

#[derive(Debug, Clone)]
pub enum DemoAction {
    OpenCityPicker(String),
    SwitchCity(Location),
    OpenSettings,
    SetHeroVisual(HeroVisualArg),
    SetTheme(ThemeArg),
    CloseSettings,
    Quit,
}

#[derive(Debug)]
pub enum AppEvent {
    Bootstrap,
    TickFrame,
    TickRefresh,
    ForceRedraw,
    Input(Event),
    FetchStarted,
    GeocodeResolved(GeocodeResolution),
    FetchSucceeded(ForecastBundle),
    FetchFailed(String),
    Demo(DemoAction),
    Quit,
}

pub fn spawn_input_task() -> impl futures::Stream<Item = Event> {
    EventStream::new().filter_map(|event| async move { event.ok() })
}

pub fn start_frame_task(tx: tokio::sync::mpsc::Sender<AppEvent>, fps: u8) {
    let fps = fps.max(15);
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(1000_u64 / u64::from(fps)));
        loop {
            ticker.tick().await;
            if tx.send(AppEvent::TickFrame).await.is_err() {
                break;
            }
        }
    });
}

pub fn start_refresh_task(tx: tokio::sync::mpsc::Sender<AppEvent>, refresh_secs: u64) {
    tokio::spawn(async move {
        let base = refresh_secs.max(10);
        loop {
            let wait_secs = {
                let mut rng = rand::rng();
                let jitter = rng.random_range(-0.1f32..0.1f32);
                ((base as f32) * (1.0 + jitter)).max(1.0)
            };
            sleep(Duration::from_secs_f32(wait_secs)).await;
            if tx.send(AppEvent::TickRefresh).await.is_err() {
                break;
            }
        }
    });
}

pub fn schedule_retry(tx: tokio::sync::mpsc::Sender<AppEvent>, delay_secs: u64) {
    tokio::spawn(async move {
        sleep(Duration::from_secs(delay_secs.max(1))).await;
        let _ = tx.send(AppEvent::TickRefresh).await;
    });
}

pub fn start_demo_task(tx: tokio::sync::mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        for (delay, action) in demo_script() {
            sleep(delay).await;
            if tx.send(AppEvent::Demo(action)).await.is_err() {
                break;
            }
        }
    });
}

fn demo_script() -> Vec<(Duration, DemoAction)> {
    let mut steps = Vec::new();
    append_demo_city_steps(&mut steps);
    append_demo_visual_steps(&mut steps);
    append_demo_theme_steps(&mut steps);
    append_demo_finish_steps(&mut steps);

    steps
}

fn append_demo_city_steps(steps: &mut Vec<(Duration, DemoAction)>) {
    for (delay_secs, query, location) in demo_city_stops() {
        push_city_demo_step(steps, delay_secs, query, location);
    }
}

fn demo_city_stops() -> [(u64, &'static str, Location); 4] {
    [
        (1, "New York", new_york_city()),
        (3, "Miami", miami_city()),
        (3, "Sydney", sydney_city()),
        (3, "Peking", peking_city()),
    ]
}

fn new_york_city() -> Location {
    demo_city(
        "New York",
        40.7128,
        -74.0060,
        "United States",
        "New York",
        "America/New_York",
    )
}

fn miami_city() -> Location {
    demo_city(
        "Miami",
        25.7617,
        -80.1918,
        "United States",
        "Florida",
        "America/New_York",
    )
}

fn sydney_city() -> Location {
    demo_city(
        "Sydney",
        -33.8688,
        151.2093,
        "Australia",
        "New South Wales",
        "Australia/Sydney",
    )
}

fn peking_city() -> Location {
    demo_city(
        "Peking",
        39.9042,
        116.4074,
        "China",
        "Beijing",
        "Asia/Shanghai",
    )
}

fn append_demo_visual_steps(steps: &mut Vec<(Duration, DemoAction)>) {
    let visual_steps = [
        (3, DemoAction::OpenSettings),
        (1, DemoAction::SetHeroVisual(HeroVisualArg::GaugeCluster)),
        (1, DemoAction::CloseSettings),
        (5, DemoAction::OpenSettings),
        (1, DemoAction::SetHeroVisual(HeroVisualArg::SkyObservatory)),
        (1, DemoAction::CloseSettings),
        (5, DemoAction::OpenSettings),
    ];
    steps.extend(
        visual_steps
            .into_iter()
            .map(|(secs, action)| (Duration::from_secs(secs), action)),
    );
}

fn append_demo_theme_steps(steps: &mut Vec<(Duration, DemoAction)>) {
    let themes = [
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
    steps.extend(
        themes
            .into_iter()
            .map(|theme| (Duration::from_secs(1), DemoAction::SetTheme(theme))),
    );
}

fn append_demo_finish_steps(steps: &mut Vec<(Duration, DemoAction)>) {
    steps.push((Duration::from_secs(1), DemoAction::CloseSettings));
    steps.push((Duration::from_secs(1), DemoAction::Quit));
}

fn push_city_demo_step(
    steps: &mut Vec<(Duration, DemoAction)>,
    open_delay_secs: u64,
    query: &str,
    location: Location,
) {
    steps.push((
        Duration::from_secs(open_delay_secs),
        DemoAction::OpenCityPicker(query.to_string()),
    ));
    steps.push((Duration::from_secs(2), DemoAction::SwitchCity(location)));
}

fn demo_city(
    name: &str,
    latitude: f64,
    longitude: f64,
    country: &str,
    admin1: &str,
    timezone: &str,
) -> Location {
    Location {
        name: name.to_string(),
        latitude,
        longitude,
        country: Some(country.to_string()),
        admin1: Some(admin1.to_string()),
        timezone: Some(timezone.to_string()),
        population: None,
    }
}

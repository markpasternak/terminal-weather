#![allow(clippy::cast_precision_loss)]

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use rand::Rng;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
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

pub fn start_refresh_task(tx: tokio::sync::mpsc::Sender<AppEvent>, refresh_secs: Arc<AtomicU64>) {
    tokio::spawn(async move {
        loop {
            let base = refresh_secs.load(Ordering::Relaxed).max(10);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::{Duration, timeout};

    #[test]
    fn demo_script_contains_expected_flow_markers() {
        let steps = demo_script();
        assert!(!steps.is_empty());

        assert!(matches!(
            steps.first(),
            Some((_, DemoAction::OpenCityPicker(query))) if query == "New York"
        ));
        assert!(matches!(steps.last(), Some((_, DemoAction::Quit))));

        let close_count = steps
            .iter()
            .filter(|(_, action)| matches!(action, DemoAction::CloseSettings))
            .count();
        assert!(close_count >= 3);
    }

    #[test]
    fn demo_city_stops_are_stable() {
        let stops = demo_city_stops();
        assert_eq!(stops.len(), 4);
        assert_eq!(stops[0].1, "New York");
        assert_eq!(stops[1].1, "Miami");
        assert_eq!(stops[2].1, "Sydney");
        assert_eq!(stops[3].1, "Peking");
        assert_eq!(stops[0].2.timezone.as_deref(), Some("America/New_York"));
    }

    #[test]
    fn push_city_demo_step_adds_open_and_switch_actions() {
        let mut steps = Vec::new();
        let location = demo_city("Test", 10.0, 20.0, "Country", "Admin", "Etc/UTC");
        push_city_demo_step(&mut steps, 3, "Test", location.clone());

        assert_eq!(steps.len(), 2);
        assert!(matches!(
            &steps[0],
            (delay, DemoAction::OpenCityPicker(query))
                if *delay == Duration::from_secs(3) && query == "Test"
        ));
        assert!(matches!(
            &steps[1],
            (delay, DemoAction::SwitchCity(loc))
                if *delay == Duration::from_secs(2) && loc.name == location.name
        ));
    }

    #[tokio::test]
    async fn schedule_retry_sends_tick_refresh() {
        let (tx, mut rx) = mpsc::channel(2);
        schedule_retry(tx, 0);
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("retry event should arrive")
            .expect("channel event");
        assert!(matches!(event, AppEvent::TickRefresh));
    }

    #[tokio::test]
    async fn start_frame_task_emits_tick_frame() {
        let (tx, mut rx) = mpsc::channel(4);
        start_frame_task(tx, 60);
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("frame event should arrive")
            .expect("channel event");
        assert!(matches!(event, AppEvent::TickFrame));
    }

    #[tokio::test]
    async fn start_frame_task_clamps_fps_to_minimum() {
        let (tx, mut rx) = mpsc::channel(4);
        start_frame_task(tx, 5);
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("frame event should arrive with clamped fps")
            .expect("channel event");
        assert!(matches!(event, AppEvent::TickFrame));
    }
}

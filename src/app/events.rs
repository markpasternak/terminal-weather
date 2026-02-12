use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use rand::Rng;
use tokio::time::{interval, sleep};

use crate::domain::weather::{ForecastBundle, GeocodeResolution, SilhouetteArt};

#[derive(Debug)]
pub enum AppEvent {
    Bootstrap,
    TickFrame,
    TickRefresh,
    Input(Event),
    FetchStarted,
    GeocodeResolved(GeocodeResolution),
    FetchSucceeded(ForecastBundle),
    FetchFailed(String),
    SilhouetteFetched {
        key: String,
        art: Option<SilhouetteArt>,
    },
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

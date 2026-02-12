use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc;

use crate::{
    app::events::{AppEvent, schedule_retry, start_frame_task, start_refresh_task},
    cli::{Cli, SilhouetteSourceArg, UnitsArg},
    data::{
        forecast::ForecastClient,
        geocode::GeocodeClient,
        silhouette::{SilhouetteClient, cache_key},
    },
    domain::weather::{
        ForecastBundle, GeocodeResolution, Location, RefreshMetadata, SilhouetteArt, Units,
        evaluate_freshness,
    },
    resilience::backoff::Backoff,
    ui::particles::ParticleEngine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    SelectingLocation,
    Ready,
    Error,
    Quit,
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
    pub particles: ParticleEngine,
    pub backoff: Backoff,
    pub fetch_in_flight: bool,
    pub last_frame_at: Instant,
    pub frame_tick: u64,
    pub animate_ui: bool,
    pub active_location_key: Option<String>,
    pub web_silhouettes: HashMap<String, SilhouetteArt>,
    pub silhouettes_in_flight: HashSet<String>,
}

impl AppState {
    pub fn new(cli: &Cli) -> Self {
        let units = match cli.units {
            UnitsArg::Celsius => Units::Celsius,
            UnitsArg::Fahrenheit => Units::Fahrenheit,
        };

        Self {
            mode: AppMode::Loading,
            running: true,
            loading_message: "Initializing...".to_string(),
            last_error: None,
            selected_location: None,
            pending_locations: Vec::new(),
            weather: None,
            refresh_meta: RefreshMetadata::default(),
            units,
            hourly_offset: 0,
            particles: ParticleEngine::new(cli.no_animation, cli.reduced_motion, cli.no_flash),
            backoff: Backoff::new(10, 300),
            fetch_in_flight: false,
            last_frame_at: Instant::now(),
            frame_tick: 0,
            animate_ui: !cli.no_animation && !cli.reduced_motion,
            active_location_key: None,
            web_silhouettes: HashMap::new(),
            silhouettes_in_flight: HashSet::new(),
        }
    }

    pub async fn handle_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            AppEvent::Bootstrap => {
                cli.validate()?;
                start_frame_task(
                    tx.clone(),
                    if cli.reduced_motion {
                        cli.fps.min(20)
                    } else {
                        cli.fps
                    },
                );
                start_refresh_task(tx.clone(), cli.refresh_interval);
                self.start_fetch(tx, cli).await?;
            }
            AppEvent::TickFrame => {
                let now = Instant::now();
                let delta = now.duration_since(self.last_frame_at);
                self.last_frame_at = now;
                self.frame_tick = self.frame_tick.saturating_add(1);

                self.particles.update(
                    self.weather.as_ref().map(|w| w.current_weather_code()),
                    self.weather.as_ref().map(|w| w.current.wind_speed_10m),
                    self.weather.as_ref().map(|w| w.current.wind_direction_10m),
                    delta,
                );
                self.refresh_meta.state = evaluate_freshness(
                    self.refresh_meta.last_success,
                    self.refresh_meta.consecutive_failures,
                );
            }
            AppEvent::TickRefresh => {
                if matches!(
                    self.mode,
                    AppMode::Ready | AppMode::Error | AppMode::Loading
                ) {
                    self.start_fetch(tx, cli).await?;
                }
            }
            AppEvent::Input(event) => self.handle_input(event, tx, cli).await?,
            AppEvent::FetchStarted => {
                self.fetch_in_flight = true;
                self.loading_message = "Fetching weather...".to_string();
                if self.weather.is_none() {
                    self.mode = AppMode::Loading;
                }
                self.refresh_meta.last_attempt = Some(chrono::Utc::now());
            }
            AppEvent::GeocodeResolved(resolution) => match resolution {
                GeocodeResolution::Selected(location) => {
                    self.selected_location = Some(location.clone());
                    self.pending_locations.clear();
                    self.fetch_forecast(tx, location).await?;
                }
                GeocodeResolution::NeedsDisambiguation(locations) => {
                    self.pending_locations = locations;
                    self.fetch_in_flight = false;
                    self.mode = AppMode::SelectingLocation;
                    self.loading_message = "Choose a location (1-5)".to_string();
                }
                GeocodeResolution::NotFound(city) => {
                    self.fetch_in_flight = false;
                    self.mode = AppMode::Error;
                    self.last_error = Some(format!("No geocoding result for {city}"));
                }
            },
            AppEvent::FetchSucceeded(bundle) => {
                let location = bundle.location.clone();
                self.fetch_in_flight = false;
                self.weather = Some(bundle);
                self.mode = AppMode::Ready;
                self.last_error = None;
                self.refresh_meta.mark_success();
                self.backoff.reset();
                self.hourly_offset = 0;
                self.active_location_key = Some(cache_key(&location));
                self.maybe_fetch_silhouette(tx, cli, &location);
            }
            AppEvent::FetchFailed(err) => {
                self.fetch_in_flight = false;
                self.last_error = Some(err);
                self.mode = AppMode::Error;
                self.refresh_meta.mark_failure();
                self.refresh_meta.state = evaluate_freshness(
                    self.refresh_meta.last_success,
                    self.refresh_meta.consecutive_failures,
                );
                let delay = self.backoff.next_delay();
                schedule_retry(tx.clone(), delay);
            }
            AppEvent::SilhouetteFetched { key, art } => {
                self.silhouettes_in_flight.remove(&key);
                if let Some(art) = art {
                    self.web_silhouettes.insert(key, art);
                }
            }
            AppEvent::Quit => {
                self.mode = AppMode::Quit;
            }
        }

        Ok(())
    }

    async fn handle_input(
        &mut self,
        event: Event,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    tx.send(AppEvent::Quit).await?;
                }
                KeyCode::Char('r') => {
                    self.start_fetch(tx, cli).await?;
                }
                KeyCode::Char('f') => {
                    self.units = Units::Fahrenheit;
                }
                KeyCode::Char('c') => {
                    self.units = Units::Celsius;
                }
                KeyCode::Left => {
                    self.hourly_offset = self.hourly_offset.saturating_sub(1);
                }
                KeyCode::Right => {
                    if let Some(bundle) = &self.weather {
                        let max_visible = 6;
                        let max_offset = bundle.hourly.len().saturating_sub(max_visible);
                        self.hourly_offset = (self.hourly_offset + 1).min(max_offset);
                    }
                }
                KeyCode::Char(digit @ '1'..='5') if self.mode == AppMode::SelectingLocation => {
                    let idx = (digit as usize) - ('1' as usize);
                    if let Some(selected) = self.pending_locations.get(idx).cloned() {
                        self.selected_location = Some(selected.clone());
                        self.pending_locations.clear();
                        self.mode = AppMode::Loading;
                        self.fetch_forecast(tx, selected).await?;
                    }
                }
                _ => {}
            },
            Event::Resize(_, _) => {
                self.particles.reset();
            }
            _ => {}
        }

        Ok(())
    }

    async fn start_fetch(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        if self.fetch_in_flight || self.mode == AppMode::SelectingLocation {
            return Ok(());
        }

        tx.send(AppEvent::FetchStarted).await?;

        if let Some(location) = self.selected_location.clone() {
            self.fetch_forecast(tx, location).await?;
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

        Ok(())
    }

    async fn fetch_forecast(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        location: Location,
    ) -> Result<()> {
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
        Ok(())
    }

    fn maybe_fetch_silhouette(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
        location: &Location,
    ) {
        if matches!(cli.silhouette_source, SilhouetteSourceArg::Local) {
            return;
        }

        let key = cache_key(location);
        if self.web_silhouettes.contains_key(&key) || self.silhouettes_in_flight.contains(&key) {
            return;
        }

        self.silhouettes_in_flight.insert(key.clone());
        let tx2 = tx.clone();
        let location = location.clone();
        tokio::spawn(async move {
            let client = SilhouetteClient::new();
            let art = client.fetch_for_location(&location).await.ok().flatten();
            let _ = tx2.send(AppEvent::SilhouetteFetched { key, art }).await;
        });
    }
}

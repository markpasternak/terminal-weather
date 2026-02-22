use super::*;

impl AppState {
    pub(crate) fn handle_sync_event(&mut self, event: AppEvent, tx: &mpsc::Sender<AppEvent>) {
        match event {
            AppEvent::TickFrame => self.handle_tick_frame(),
            AppEvent::FetchStarted => self.handle_fetch_started(),
            AppEvent::GeocodeResolved(resolution) => {
                self.handle_geocode_resolved(tx, resolution);
            }
            AppEvent::FetchSucceeded(bundle) => self.handle_fetch_succeeded(bundle),
            AppEvent::FetchFailed(err) => self.handle_fetch_failed(tx, err),
            AppEvent::Quit => self.mode = AppMode::Quit,
            _ => {}
        }
    }

    pub(crate) async fn handle_bootstrap(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        cli.validate()?;
        start_frame_task(tx.clone(), cli.fps);
        start_refresh_task(tx.clone(), self.refresh_interval_secs_runtime.clone());
        if cli.demo {
            start_demo_task(tx.clone());
        }
        self.start_fetch(tx, cli).await
    }

    pub(crate) fn handle_tick_frame(&mut self) {
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

    pub(crate) async fn handle_tick_refresh(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if matches!(
            self.mode,
            AppMode::Ready | AppMode::Error | AppMode::Loading
        ) {
            self.start_fetch(tx, cli).await?;
        }
        Ok(())
    }

    pub(crate) fn handle_fetch_started(&mut self) {
        self.fetch_in_flight = true;
        self.loading_message = "Fetching weather...".to_string();
        if self.weather.is_none() {
            self.mode = AppMode::Loading;
        }
        self.refresh_meta.last_attempt = Some(chrono::Utc::now());
        self.refresh_meta.clear_retry();
    }

    pub(crate) fn handle_geocode_resolved(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        resolution: GeocodeResolution,
    ) {
        match resolution {
            GeocodeResolution::Selected(location) => {
                self.switch_to_location(tx, location);
            }
            GeocodeResolution::NeedsDisambiguation(locations) => {
                self.pending_locations = locations;
                self.fetch_in_flight = false;
                self.mode = AppMode::SelectingLocation;
                self.loading_message = "Choose a location (1-5)".to_string();
                self.city_picker_open = false;
                self.city_status =
                    Some("Ambiguous results: choose a matching location".to_string());
            }
            GeocodeResolution::NotFound(city) => {
                self.fetch_in_flight = false;
                self.mode = AppMode::Error;
                self.last_error = Some(format!("No geocoding result for {city}"));
                self.city_status = Some(format!("No results for '{city}'"));
            }
        }
    }

    pub(crate) fn handle_fetch_succeeded(&mut self, bundle: ForecastBundle) {
        let location = bundle.location.clone();
        let key: LocationKey = (&location).into();
        self.forecast_cache.put(key, bundle.clone());
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

    pub(crate) fn handle_fetch_failed(&mut self, tx: &mpsc::Sender<AppEvent>, err: String) {
        self.fetch_in_flight = false;
        self.last_error = Some(err);
        self.mode = AppMode::Error;
        self.city_status =
            Some("Failed to fetch weather; keeping last successful data".to_string());
        self.refresh_meta.mark_failure();
        self.refresh_meta.state = evaluate_freshness(
            self.refresh_meta.last_success,
            self.refresh_meta.consecutive_failures,
        );
        let delay = self.backoff.next_delay();
        self.refresh_meta.schedule_retry_in(delay);
        schedule_retry(tx.clone(), delay);
    }
}

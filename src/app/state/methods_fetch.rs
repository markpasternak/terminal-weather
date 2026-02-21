use super::*;

impl AppState {
    pub(crate) async fn start_fetch(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if self.fetch_blocked() {
            return Ok(());
        }
        tx.send(AppEvent::FetchStarted).await?;
        if self.try_fetch_existing_location(tx) {
            return Ok(());
        }
        if Self::try_fetch_coords(tx, cli).await? {
            return Ok(());
        }
        if Self::should_auto_lookup(cli) {
            self.start_auto_location_lookup(tx, cli.country_code.clone());
        } else {
            Self::start_city_lookup(tx, cli.default_city(), cli.country_code.clone());
        }
        Ok(())
    }

    fn fetch_blocked(&self) -> bool {
        self.fetch_in_flight || self.mode == AppMode::SelectingLocation
    }

    fn should_auto_lookup(cli: &Cli) -> bool {
        cli.city.is_none() && cli.lat.is_none()
    }

    pub(crate) fn try_fetch_existing_location(&self, tx: &mpsc::Sender<AppEvent>) -> bool {
        if let Some(location) = self.selected_location.clone() {
            self.fetch_forecast(tx, location);
            return true;
        }
        false
    }

    pub(crate) async fn try_fetch_coords(tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<bool> {
        if let (Some(lat), Some(lon)) = (cli.lat, cli.lon) {
            let location = Location::from_coords(lat, lon);
            tx.send(AppEvent::GeocodeResolved(GeocodeResolution::Selected(
                location,
            )))
            .await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn start_auto_location_lookup(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        country_code: Option<String>,
    ) {
        self.loading_message = "Detecting location...".to_string();
        let tx2 = tx.clone();
        tokio::spawn(async move {
            if let Some(location) = crate::data::geoip::detect_location().await {
                let _ = tx2
                    .send(AppEvent::GeocodeResolved(GeocodeResolution::Selected(
                        location,
                    )))
                    .await;
                return;
            }
            resolve_city_with_geocoder(tx2, "Stockholm".to_string(), country_code).await;
        });
    }

    pub(crate) fn start_city_lookup(
        tx: &mpsc::Sender<AppEvent>,
        city: String,
        country_code: Option<String>,
    ) {
        let tx2 = tx.clone();
        tokio::spawn(async move {
            resolve_city_with_geocoder(tx2, city, country_code).await;
        });
    }

    pub(crate) fn fetch_forecast(&self, tx: &mpsc::Sender<AppEvent>, location: Location) {
        let client = self.build_forecast_client();
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

    fn build_forecast_client(&self) -> ForecastClient {
        match (&self.forecast_url_override, &self.air_quality_url_override) {
            (Some(forecast_url), Some(air_quality_url)) => {
                ForecastClient::with_urls(forecast_url.clone(), air_quality_url.clone())
            }
            (Some(forecast_url), None) => ForecastClient::with_base_url(forecast_url.clone()),
            (None, Some(air_quality_url)) => {
                ForecastClient::new().with_air_quality_url(air_quality_url.clone())
            }
            (None, None) => ForecastClient::new(),
        }
    }

    pub(crate) async fn handle_demo_action(
        &mut self,
        action: DemoAction,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<()> {
        if matches!(action, DemoAction::Quit) {
            tx.send(AppEvent::Quit).await?;
            return Ok(());
        }
        self.apply_demo_action(action, tx);
        Ok(())
    }

    fn apply_demo_action(&mut self, action: DemoAction, tx: &mpsc::Sender<AppEvent>) {
        match action {
            DemoAction::OpenCityPicker(query) => self.demo_open_city_picker(&query),
            DemoAction::SwitchCity(location) => self.demo_switch_city(tx, location),
            DemoAction::OpenSettings => self.demo_open_settings(),
            DemoAction::SetHeroVisual(visual) => self.demo_set_hero_visual(visual),
            DemoAction::SetTheme(theme) => self.demo_set_theme(theme),
            DemoAction::CloseSettings => self.settings_open = false,
            DemoAction::Quit => {}
        }
    }

    pub(crate) fn demo_open_city_picker(&mut self, query: &str) {
        self.settings_open = false;
        self.city_picker_open = true;
        self.city_query.clear();
        self.city_query.push_str(query);
        self.city_history_selected = 0;
        self.city_status = Some(format!("Demo: search for {query}"));
    }

    pub(crate) fn demo_switch_city(&mut self, tx: &mpsc::Sender<AppEvent>, location: Location) {
        self.settings_open = false;
        self.city_picker_open = true;
        self.city_status = Some(format!("Demo: selected {}", location.display_name()));
        self.city_query.clear();
        self.city_picker_open = false;
        self.switch_to_location(tx, location);
    }

    pub(crate) fn demo_open_settings(&mut self) {
        self.city_picker_open = false;
        self.settings_open = true;
        self.settings_selected = SettingsSelection::HeroVisual;
    }

    pub(crate) fn demo_set_hero_visual(&mut self, visual: HeroVisualArg) {
        self.settings_open = true;
        self.settings_selected = SettingsSelection::HeroVisual;
        if self.settings.hero_visual != visual {
            self.settings.hero_visual = visual;
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    pub(crate) fn demo_set_theme(&mut self, theme: ThemeArg) {
        self.settings_open = true;
        self.settings_selected = SettingsSelection::Theme;
        if self.settings.theme != theme {
            self.settings.theme = theme;
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }
}

async fn resolve_city_with_geocoder(
    tx: mpsc::Sender<AppEvent>,
    city: String,
    country_code: Option<String>,
) {
    let geocoder = GeocodeClient::new();
    match geocoder.resolve(city, country_code).await {
        Ok(resolution) => {
            let _ = tx.send(AppEvent::GeocodeResolved(resolution)).await;
        }
        Err(err) => {
            let _ = tx.send(AppEvent::FetchFailed(err.to_string())).await;
        }
    }
}

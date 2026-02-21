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
        if self.try_fetch_existing_location(tx).await {
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

    pub(crate) async fn try_fetch_existing_location(&self, tx: &mpsc::Sender<AppEvent>) -> bool {
        let geocoder = GeocodeClient::new();
        self.try_fetch_existing_location_with_geocoder(tx, &geocoder)
            .await
    }

    pub(crate) async fn try_fetch_existing_location_with_geocoder(
        &self,
        tx: &mpsc::Sender<AppEvent>,
        geocoder: &GeocodeClient,
    ) -> bool {
        if let Some(location) = self.selected_location.clone() {
            let location = resolve_saved_location_with_reverse_geocoder(geocoder, location).await;
            self.fetch_forecast(tx, location);
            return true;
        }
        false
    }

    pub(crate) async fn try_fetch_coords(tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<bool> {
        let geocoder = GeocodeClient::new();
        Self::try_fetch_coords_with_geocoder(tx, cli, &geocoder).await
    }

    pub(crate) async fn try_fetch_coords_with_geocoder(
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
        geocoder: &GeocodeClient,
    ) -> Result<bool> {
        if let (Some(lat), Some(lon)) = (cli.lat, cli.lon) {
            let location = resolve_coords_with_reverse_geocoder(geocoder, lat, lon).await;
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

async fn resolve_coords_with_reverse_geocoder(
    geocoder: &GeocodeClient,
    lat: f64,
    lon: f64,
) -> Location {
    match geocoder.reverse_resolve(lat, lon).await {
        Ok(Some(location)) => location,
        Ok(None) | Err(_) => Location::from_coords(lat, lon),
    }
}

async fn resolve_saved_location_with_reverse_geocoder(
    geocoder: &GeocodeClient,
    location: Location,
) -> Location {
    if !is_coordinate_label(&location) {
        return location;
    }
    match geocoder
        .reverse_resolve(location.latitude, location.longitude)
        .await
    {
        Ok(Some(resolved)) => resolved,
        Ok(None) | Err(_) => location,
    }
}

fn is_coordinate_label(location: &Location) -> bool {
    location.name.trim() == format!("{:.4}, {:.4}", location.latitude, location.longitude)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{HeroVisualArg, ThemeArg};
    use tokio::sync::mpsc;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn test_state() -> AppState {
        AppState::new(&crate::test_support::state_test_cli())
    }

    #[test]
    fn fetch_blocked_when_in_flight_or_selecting_location() {
        let mut state = test_state();
        state.fetch_in_flight = true;
        assert!(state.fetch_blocked());

        state.fetch_in_flight = false;
        state.mode = AppMode::SelectingLocation;
        assert!(state.fetch_blocked());

        state.mode = AppMode::Ready;
        assert!(!state.fetch_blocked());
    }

    #[test]
    fn should_auto_lookup_only_without_city_or_lat() {
        let mut cli = crate::test_support::state_test_cli();
        assert!(AppState::should_auto_lookup(&cli));

        cli.city = Some("Berlin".to_string());
        assert!(!AppState::should_auto_lookup(&cli));

        let mut cli = crate::test_support::state_test_cli();
        cli.lat = Some(59.3);
        assert!(!AppState::should_auto_lookup(&cli));
    }

    #[tokio::test]
    async fn try_fetch_coords_emits_selected_location_when_present() {
        let mut cli = crate::test_support::state_test_cli();
        cli.lat = Some(59.3293);
        cli.lon = Some(18.0686);
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "address": {
                "city": "Stockholm",
                "state": "Stockholm County",
                "country": "Sweden"
            }
        });
        Mock::given(method("GET"))
            .and(path("/v1/reverse"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;
        let geocoder = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));

        let (tx, mut rx) = mpsc::channel(2);
        let handled = AppState::try_fetch_coords_with_geocoder(&tx, &cli, &geocoder)
            .await
            .expect("coords should be handled");

        assert!(handled);
        let event = rx.recv().await.expect("event");
        assert!(matches!(
            event,
            AppEvent::GeocodeResolved(GeocodeResolution::Selected(location))
            if location.name == "Stockholm"
            && (location.latitude - 59.3293).abs() < f64::EPSILON
        ));
    }

    #[tokio::test]
    async fn try_fetch_coords_falls_back_when_reverse_geocode_fails() {
        let mut cli = crate::test_support::state_test_cli();
        cli.lat = Some(59.3293);
        cli.lon = Some(18.0686);
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/reverse"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let geocoder = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));

        let (tx, mut rx) = mpsc::channel(2);
        let handled = AppState::try_fetch_coords_with_geocoder(&tx, &cli, &geocoder)
            .await
            .expect("coords should be handled");

        assert!(handled);
        let event = rx.recv().await.expect("event");
        assert!(matches!(
            event,
            AppEvent::GeocodeResolved(GeocodeResolution::Selected(location))
            if location.name == "59.3293, 18.0686"
        ));
    }

    #[tokio::test]
    async fn resolve_saved_location_updates_coordinate_label_when_reverse_available() {
        let server = MockServer::start().await;
        let payload = serde_json::json!({
            "address": {
                "city": "Stockholm",
                "state": "Stockholm County",
                "country": "Sweden"
            }
        });
        Mock::given(method("GET"))
            .and(path("/v1/reverse"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;
        let geocoder = GeocodeClient::with_base_url(format!("{}/v1/search", server.uri()));

        let location = resolve_saved_location_with_reverse_geocoder(
            &geocoder,
            Location::from_coords(59.3293, 18.0686),
        )
        .await;

        assert_eq!(location.name, "Stockholm");
    }

    #[tokio::test]
    async fn resolve_saved_location_keeps_named_location_without_reverse_lookup() {
        let geocoder = GeocodeClient::with_base_url("http://127.0.0.1:9");
        let named = crate::test_support::stockholm_location();

        let location = resolve_saved_location_with_reverse_geocoder(&geocoder, named.clone()).await;

        assert_eq!(location.name, named.name);
        assert_eq!(location.country, named.country);
    }

    #[tokio::test]
    async fn try_fetch_coords_returns_false_without_complete_pair() {
        let mut cli = crate::test_support::state_test_cli();
        cli.lat = Some(59.3293);
        cli.lon = None;
        let (tx, mut rx) = mpsc::channel(2);
        let handled = AppState::try_fetch_coords(&tx, &cli)
            .await
            .expect("coords path should succeed");
        assert!(!handled);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn build_forecast_client_honors_override_combinations() {
        let mut state = test_state();
        state.forecast_url_override = Some("https://example.test/forecast".to_string());
        state.air_quality_url_override = Some("https://example.test/aq".to_string());
        let both = format!("{:?}", state.build_forecast_client());
        assert!(both.contains("https://example.test/forecast"));
        assert!(both.contains("https://example.test/aq"));

        state.air_quality_url_override = None;
        let forecast_only = format!("{:?}", state.build_forecast_client());
        assert!(forecast_only.contains("https://example.test/forecast"));

        state.forecast_url_override = None;
        state.air_quality_url_override = Some("https://example.test/aq2".to_string());
        let aq_only = format!("{:?}", state.build_forecast_client());
        assert!(aq_only.contains("https://example.test/aq2"));
    }

    #[tokio::test]
    async fn handle_demo_action_quit_emits_quit_event() {
        let mut state = test_state();
        let (tx, mut rx) = mpsc::channel(2);
        state
            .handle_demo_action(DemoAction::Quit, &tx)
            .await
            .expect("quit should be handled");
        assert!(matches!(rx.recv().await, Some(AppEvent::Quit)));
    }

    #[test]
    fn demo_open_city_picker_sets_expected_ui_state() {
        let mut state = test_state();
        state.settings_open = true;
        state.demo_open_city_picker("Tokyo");
        assert!(!state.settings_open);
        assert!(state.city_picker_open);
        assert_eq!(state.city_query, "Tokyo");
        assert_eq!(state.city_history_selected, 0);
    }

    #[test]
    fn demo_set_hero_visual_changes_only_when_needed() {
        let mut state = test_state();
        state.settings.hero_visual = HeroVisualArg::AtmosCanvas;
        state.demo_set_hero_visual(HeroVisualArg::GaugeCluster);
        assert_eq!(state.settings.hero_visual, HeroVisualArg::GaugeCluster);
        let prev = state.settings.hero_visual;
        state.demo_set_hero_visual(HeroVisualArg::GaugeCluster);
        assert_eq!(state.settings.hero_visual, prev);
    }

    #[test]
    fn demo_set_theme_changes_only_when_needed() {
        let mut state = test_state();
        state.settings.theme = ThemeArg::Auto;
        state.demo_set_theme(ThemeArg::Nord);
        assert_eq!(state.settings.theme, ThemeArg::Nord);
        let prev = state.settings.theme;
        state.demo_set_theme(ThemeArg::Nord);
        assert_eq!(state.settings.theme, prev);
    }

    #[tokio::test]
    async fn try_fetch_existing_location_returns_false_without_selection() {
        let state = test_state();
        let (tx, _rx) = mpsc::channel(2);
        assert!(!state.try_fetch_existing_location(&tx).await);
    }

    #[tokio::test]
    async fn try_fetch_existing_location_returns_true_with_selection() {
        let mut state = test_state();
        state.selected_location = Some(crate::test_support::stockholm_location());
        state.forecast_url_override = Some("http://127.0.0.1:1".to_string());
        state.air_quality_url_override = Some("http://127.0.0.1:1".to_string());
        let (tx, _rx) = mpsc::channel(2);
        assert!(state.try_fetch_existing_location(&tx).await);
    }
}

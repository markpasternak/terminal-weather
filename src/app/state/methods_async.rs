use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyCommand {
    Quit,
    OpenSettings,
    OpenCityPicker,
    Refresh,
    SetFahrenheit,
    SetCelsius,
    CycleHourlyView,
}

impl AppState {
    pub async fn handle_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if event_is_async(&event) {
            self.handle_async_event(event, tx, cli).await?;
        } else {
            self.handle_sync_event(event, tx);
        }

        Ok(())
    }

    pub(crate) async fn handle_async_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if self.handle_async_system_event(&event, tx, cli).await? {
            return Ok(());
        }
        self.handle_async_user_event(event, tx, cli).await
    }

    async fn handle_async_system_event(
        &mut self,
        event: &AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        match event {
            AppEvent::Bootstrap => {
                self.handle_bootstrap(tx, cli).await?;
                Ok(true)
            }
            AppEvent::TickRefresh => {
                self.handle_tick_refresh(tx, cli).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn handle_async_user_event(
        &mut self,
        event: AppEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            AppEvent::Input(input) => self.handle_input(input, tx, cli).await?,
            AppEvent::Demo(action) => self.handle_demo_action(action, tx).await?,
            _ => {}
        }
        Ok(())
    }

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
        let frame_fps = match self.settings.motion {
            MotionSetting::Full => cli.fps,
            MotionSetting::Reduced => cli.fps.min(20),
            MotionSetting::Off => 15,
        };
        start_frame_task(tx.clone(), frame_fps);
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
                self.selected_location = Some(location.clone());
                self.pending_locations.clear();
                Self::fetch_forecast(tx, location);
            }
            GeocodeResolution::NeedsDisambiguation(locations) => {
                self.pending_locations = locations;
                self.fetch_in_flight = false;
                self.mode = AppMode::SelectingLocation;
                self.loading_message = "Choose a location (1-5)".to_string();
                self.city_picker_open = false;
            }
            GeocodeResolution::NotFound(city) => {
                self.fetch_in_flight = false;
                self.mode = AppMode::Error;
                self.last_error = Some(format!("No geocoding result for {city}"));
            }
        }
    }

    pub(crate) fn handle_fetch_succeeded(&mut self, bundle: ForecastBundle) {
        let location = bundle.location.clone();
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
        self.city_status = Some("Search failed; keeping last successful weather".to_string());
        self.refresh_meta.mark_failure();
        self.refresh_meta.state = evaluate_freshness(
            self.refresh_meta.last_success,
            self.refresh_meta.consecutive_failures,
        );
        let delay = self.backoff.next_delay();
        self.refresh_meta.schedule_retry_in(delay);
        schedule_retry(tx.clone(), delay);
    }

    pub(crate) async fn handle_input(
        &mut self,
        event: Event,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key_press(key, tx, cli).await?;
            }
            Event::Resize(width, _) => {
                self.viewport_width = width;
                self.particles.reset();
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) async fn handle_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if self.handle_control_shortcuts(key, tx).await? {
            return Ok(());
        }
        if self.handle_modal_key_press(key, tx, cli).await? {
            return Ok(());
        }
        self.handle_main_key_press(key, tx, cli).await
    }

    pub(crate) async fn handle_control_shortcuts(
        &self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<bool> {
        if matches!(key.code, KeyCode::Char('c' | 'C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            tx.send(AppEvent::Quit).await?;
            return Ok(true);
        }
        if matches!(key.code, KeyCode::Char('l' | 'L'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            tx.send(AppEvent::ForceRedraw).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) async fn handle_modal_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        if self.settings_open {
            self.handle_settings_key(key.code, tx, cli).await?;
            return Ok(true);
        }
        if self.city_picker_open {
            self.handle_city_picker_key(key, tx, cli);
            return Ok(true);
        }
        if self.help_open {
            self.handle_help_key(key, tx).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) async fn handle_main_key_press(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if self.handle_global_main_key(key.code, tx).await? {
            return Ok(());
        }
        if self.handle_hourly_navigation_key(key.code) {
            return Ok(());
        }
        if self.try_select_pending_location(key.code, tx) {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Char(_)) {
            self.handle_char_command(key, tx, cli).await?;
        }
        Ok(())
    }

    pub(crate) async fn handle_global_main_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<bool> {
        match code {
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.open_help_overlay();
                Ok(true)
            }
            KeyCode::Esc => {
                tx.send(AppEvent::Quit).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn handle_hourly_navigation_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Left => {
                self.move_hourly_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_hourly_cursor_right();
                true
            }
            _ => false,
        }
    }

    pub(crate) fn try_select_pending_location(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
    ) -> bool {
        if let KeyCode::Char(digit @ '1'..='5') = code
            && self.mode == AppMode::SelectingLocation
        {
            self.select_pending_location(tx, digit);
            return true;
        }
        false
    }

    pub(crate) async fn handle_char_command(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        let Some(cmd) = command_char(key) else {
            return Ok(false);
        };
        let Some(action) = command_from_char(cmd) else {
            return Ok(false);
        };
        self.execute_key_command(action, tx, cli).await
    }

    async fn execute_key_command(
        &mut self,
        action: KeyCommand,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<bool> {
        if matches!(action, KeyCommand::Quit) {
            self.command_quit(tx).await?;
            return Ok(true);
        }
        if matches!(action, KeyCommand::Refresh) {
            self.command_refresh(tx, cli).await?;
            return Ok(true);
        }
        self.execute_sync_key_command(action);
        Ok(true)
    }

    fn execute_sync_key_command(&mut self, action: KeyCommand) {
        match action {
            KeyCommand::OpenSettings => self.command_open_settings(),
            KeyCommand::OpenCityPicker => self.command_open_city_picker(),
            KeyCommand::SetFahrenheit => self.set_units(Units::Fahrenheit),
            KeyCommand::SetCelsius => self.set_units(Units::Celsius),
            KeyCommand::CycleHourlyView => self.command_cycle_hourly_view(),
            KeyCommand::Quit | KeyCommand::Refresh => {}
        }
    }

    async fn command_quit(&self, tx: &mpsc::Sender<AppEvent>) -> Result<()> {
        tx.send(AppEvent::Quit).await?;
        Ok(())
    }

    fn command_open_settings(&mut self) {
        if self.mode != AppMode::SelectingLocation {
            self.open_settings_panel();
        }
    }

    fn command_open_city_picker(&mut self) {
        if self.mode != AppMode::SelectingLocation {
            self.open_city_picker();
        }
    }

    async fn command_refresh(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) -> Result<()> {
        self.start_fetch(tx, cli).await?;
        Ok(())
    }

    fn command_cycle_hourly_view(&mut self) {
        self.settings.hourly_view = cycle(&HOURLY_VIEW_OPTIONS, self.hourly_view_mode, 1);
        self.apply_runtime_settings();
        self.persist_settings();
    }

    pub(crate) fn select_pending_location(&mut self, tx: &mpsc::Sender<AppEvent>, digit: char) {
        let idx = (digit as usize) - ('1' as usize);
        if let Some(selected) = self.pending_locations.get(idx).cloned() {
            self.selected_location = Some(selected.clone());
            self.pending_locations.clear();
            self.mode = AppMode::Loading;
            Self::fetch_forecast(tx, selected);
        }
    }

    pub(crate) fn open_help_overlay(&mut self) {
        self.help_open = true;
        self.settings_open = false;
        self.city_picker_open = false;
    }

    pub(crate) fn open_settings_panel(&mut self) {
        self.city_picker_open = false;
        self.help_open = false;
        self.settings_open = true;
        self.settings_selected = SettingsSelection::Units;
    }

    pub(crate) fn open_city_picker(&mut self) {
        self.settings_open = false;
        self.help_open = false;
        self.city_picker_open = true;
        self.city_query.clear();
        self.city_history_selected = 0;
        self.city_status = Some("Type a city and press Enter, or pick from history".to_string());
    }

    pub(crate) fn set_units(&mut self, units: Units) {
        if self.settings.units != units {
            self.settings.units = units;
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    pub(crate) fn move_hourly_cursor_left(&mut self) {
        if self.hourly_cursor > 0 {
            self.hourly_cursor -= 1;
            if self.hourly_cursor < self.hourly_offset {
                self.hourly_offset = self.hourly_cursor;
            }
        }
    }

    pub(crate) fn move_hourly_cursor_right(&mut self) {
        if let Some(bundle) = &self.weather {
            let max = bundle.hourly.len().saturating_sub(1);
            if self.hourly_cursor < max {
                self.hourly_cursor += 1;
                let visible = visible_hour_count(self.viewport_width);
                if self.hourly_cursor >= self.hourly_offset + visible {
                    self.hourly_offset =
                        self.hourly_cursor.saturating_sub(visible.saturating_sub(1));
                }
            }
        }
    }
}

fn command_from_char(cmd: char) -> Option<KeyCommand> {
    const KEY_COMMANDS: [(char, KeyCommand); 7] = [
        ('q', KeyCommand::Quit),
        ('s', KeyCommand::OpenSettings),
        ('l', KeyCommand::OpenCityPicker),
        ('r', KeyCommand::Refresh),
        ('f', KeyCommand::SetFahrenheit),
        ('c', KeyCommand::SetCelsius),
        ('v', KeyCommand::CycleHourlyView),
    ];

    KEY_COMMANDS
        .iter()
        .find_map(|(target, action)| (*target == cmd).then_some(*action))
}

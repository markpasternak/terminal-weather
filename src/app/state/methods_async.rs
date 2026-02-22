use super::*;
use clap::ValueEnum;

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
        if self.command_bar.open {
            self.handle_command_bar_key(key, tx, cli).await?;
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
        if !key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(false);
        }
        match key.code {
            KeyCode::Char('c' | 'C') => {
                tx.send(AppEvent::Quit).await?;
                Ok(true)
            }
            KeyCode::Char('l' | 'L') => {
                tx.send(AppEvent::ForceRedraw).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn open_modal_if_available(&mut self, open_modal: fn(&mut AppState)) {
        if self.mode != AppMode::SelectingLocation {
            open_modal(self);
        }
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
        if self.settings.command_bar_enabled && matches!(key.code, KeyCode::Char(':')) {
            self.command_bar.open();
            return Ok(());
        }
        if self.handle_global_main_key(key.code, tx).await? {
            return Ok(());
        }
        if self.handle_panel_focus_key(key.code) {
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
        match action {
            KeyCommand::Quit => self.command_quit(tx).await?,
            KeyCommand::Refresh => self.command_refresh(tx, cli).await?,
            _ => self.execute_sync_key_command(action),
        }
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
        self.open_modal_if_available(Self::open_settings_panel);
    }

    fn command_open_city_picker(&mut self) {
        self.open_modal_if_available(Self::open_city_picker);
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
            self.fetch_forecast(tx, selected);
        }
    }

    pub(crate) fn open_help_overlay(&mut self) {
        self.help_open = true;
        self.settings_open = false;
        self.city_picker_open = false;
        self.command_bar.close();
    }

    pub(crate) fn open_settings_panel(&mut self) {
        self.city_picker_open = false;
        self.help_open = false;
        self.settings_open = true;
        self.settings_selected = SettingsSelection::default();
        self.command_bar.close();
    }

    pub(crate) fn open_city_picker(&mut self) {
        self.settings_open = false;
        self.help_open = false;
        self.city_picker_open = true;
        self.city_query.clear();
        self.city_history_selected = 0;
        self.city_status = Some("Type a city and press Enter, or pick from history".to_string());
        self.command_bar.close();
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

    async fn handle_command_bar_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command_bar.close();
            }
            KeyCode::Backspace => {
                if self.command_bar.buffer.len() > 1 {
                    self.command_bar.buffer.pop();
                }
            }
            KeyCode::Enter => {
                self.execute_command_bar(tx, cli).await;
            }
            KeyCode::Char(ch) => {
                if key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
                {
                    return Ok(());
                }
                self.command_bar.buffer.push(ch);
                self.command_bar.parse_error = None;
            }
            _ => {}
        }
        Ok(())
    }

    async fn execute_command_bar(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) {
        let raw = self
            .command_bar
            .buffer
            .trim()
            .trim_start_matches(':')
            .to_string();
        if raw.is_empty() {
            self.command_bar.close();
            return;
        }
        match self.run_command(&raw, tx, cli).await {
            Ok(()) => self.command_bar.close(),
            Err(err) => self.command_bar.parse_error = Some(err),
        }
    }

    async fn run_command(
        &mut self,
        command: &str,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> std::result::Result<(), String> {
        let action = parse_command_action(command)?;
        self.execute_command_action(action, tx, cli).await
    }

    async fn execute_command_action(
        &mut self,
        action: CommandAction,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> std::result::Result<(), String> {
        match action {
            CommandAction::Refresh => self
                .start_fetch(tx, cli)
                .await
                .map_err(|err| format!("refresh failed: {err}"))?,
            CommandAction::Quit => tx
                .send(AppEvent::Quit)
                .await
                .map_err(|err| format!("quit failed: {err}"))?,
            CommandAction::Units(units) => self.set_units(units),
            CommandAction::View(mode) => {
                self.settings.hourly_view = mode;
                self.apply_runtime_settings();
                self.persist_settings();
            }
            CommandAction::Theme(theme) => {
                self.settings.theme = theme;
                self.apply_runtime_settings();
                self.persist_settings();
            }
            CommandAction::City(query) => {
                self.start_city_search(tx, query, cli.country_code.clone());
            }
        }

        Ok(())
    }

    pub(crate) fn handle_panel_focus_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Tab => {
                self.panel_focus = self.panel_focus.next();
                true
            }
            KeyCode::BackTab => {
                self.panel_focus = self.panel_focus.previous();
                true
            }
            _ => false,
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

fn parse_units_command(value: &str) -> Option<Units> {
    if value.eq_ignore_ascii_case("c") || value.eq_ignore_ascii_case("celsius") {
        Some(Units::Celsius)
    } else if value.eq_ignore_ascii_case("f") || value.eq_ignore_ascii_case("fahrenheit") {
        Some(Units::Fahrenheit)
    } else {
        None
    }
}

fn parse_hourly_view_command(value: &str) -> Option<HourlyViewMode> {
    if value.eq_ignore_ascii_case("table") {
        Some(HourlyViewMode::Table)
    } else if value.eq_ignore_ascii_case("hybrid") {
        Some(HourlyViewMode::Hybrid)
    } else if value.eq_ignore_ascii_case("chart") {
        Some(HourlyViewMode::Chart)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandAction {
    Refresh,
    Quit,
    Units(Units),
    View(HourlyViewMode),
    Theme(ThemeArg),
    City(String),
}

fn parse_command_action(command: &str) -> std::result::Result<CommandAction, String> {
    let mut parts = command.split_whitespace();
    let Some(verb) = parts.next().map(str::to_ascii_lowercase) else {
        return Ok(CommandAction::Refresh);
    };
    let rest: Vec<&str> = parts.collect();
    match verb.as_str() {
        "refresh" => Ok(CommandAction::Refresh),
        "quit" => Ok(CommandAction::Quit),
        "units" => cmd_units(&rest),
        "view" => cmd_view(&rest),
        "theme" => cmd_theme(&rest),
        "city" => cmd_city(&rest),
        _ => Err(format!("unknown command: {verb}")),
    }
}

fn cmd_units(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :units c|f".to_string())?;
    parse_units_command(value)
        .map(CommandAction::Units)
        .ok_or_else(|| "usage: :units c|f".to_string())
}

fn cmd_view(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :view table|hybrid|chart".to_string())?;
    parse_hourly_view_command(value)
        .map(CommandAction::View)
        .ok_or_else(|| "usage: :view table|hybrid|chart".to_string())
}

fn cmd_theme(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let value = args
        .first()
        .ok_or_else(|| "usage: :theme <name>".to_string())?;
    ThemeArg::from_str(value, true)
        .map(CommandAction::Theme)
        .map_err(|_| format!("unknown theme: {value}"))
}

fn cmd_city(args: &[&str]) -> std::result::Result<CommandAction, String> {
    let query = args.join(" ");
    if query.trim().is_empty() {
        return Err("usage: :city <name>".to_string());
    }
    Ok(CommandAction::City(query))
}

#[cfg(test)]
mod tests;

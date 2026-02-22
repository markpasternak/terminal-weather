use super::*;

mod command_bar;
mod command_parse;
mod lifecycle;
use command_parse::{KeyCommand, command_from_char};

#[cfg(test)]
use command_parse::{parse_hourly_view_command, parse_units_command};

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
        let code = key.code;
        if self.try_open_command_bar(code) {
            return Ok(());
        }
        if self.handle_main_navigation_key(code, tx).await? {
            return Ok(());
        }
        self.handle_main_char_key(key, tx, cli).await
    }

    fn try_open_command_bar(&mut self, code: KeyCode) -> bool {
        if self.settings.command_bar_enabled && matches!(code, KeyCode::Char(':')) {
            self.command_bar.open();
            return true;
        }
        false
    }

    async fn handle_main_navigation_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<bool> {
        if self.handle_global_main_key(code, tx).await? {
            return Ok(true);
        }
        let handled = self.handle_panel_focus_key(code)
            || self.handle_hourly_navigation_key(code)
            || self.try_select_pending_location(code, tx);
        Ok(handled)
    }

    async fn handle_main_char_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
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

#[cfg(test)]
mod tests;

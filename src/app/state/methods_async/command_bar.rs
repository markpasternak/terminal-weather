use super::{
    command_parse::{CommandAction, parse_command_action},
    *,
};

impl AppState {
    pub(super) async fn handle_command_bar_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command_bar.close();
            }
            KeyCode::Backspace if self.command_bar.buffer.len() > 1 => {
                self.command_bar.buffer.pop();
            }
            KeyCode::Enter => {
                self.execute_command_bar(tx, cli).await;
            }
            KeyCode::Char(ch) => {
                self.push_command_bar_char(ch, key.modifiers);
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) async fn execute_command_bar(&mut self, tx: &mpsc::Sender<AppEvent>, cli: &Cli) {
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
        if self.execute_command_action_async(&action, tx, cli).await? {
            return Ok(());
        }
        self.execute_command_action_sync(action, tx, cli);
        Ok(())
    }

    async fn execute_command_action_async(
        &mut self,
        action: &CommandAction,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> std::result::Result<bool, String> {
        match action {
            CommandAction::Refresh => {
                self.command_action_refresh(tx, cli).await?;
                Ok(true)
            }
            CommandAction::Quit => {
                self.command_action_quit(tx).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn execute_command_action_sync(
        &mut self,
        action: CommandAction,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) {
        match action {
            CommandAction::Units(units) => self.set_units(units),
            CommandAction::View(mode) => self.command_action_set_view(mode),
            CommandAction::Theme(theme) => self.command_action_set_theme(theme),
            CommandAction::City(query) => {
                self.start_city_search(tx, query, cli.country_code.clone());
            }
            CommandAction::Refresh | CommandAction::Quit => {}
        }
    }

    async fn command_action_refresh(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> std::result::Result<(), String> {
        self.start_fetch(tx, cli)
            .await
            .map_err(|err| format!("refresh failed: {err}"))
    }

    async fn command_action_quit(
        &self,
        tx: &mpsc::Sender<AppEvent>,
    ) -> std::result::Result<(), String> {
        tx.send(AppEvent::Quit)
            .await
            .map_err(|err| format!("quit failed: {err}"))
    }

    fn command_action_set_view(&mut self, mode: crate::app::state::HourlyViewMode) {
        self.settings.hourly_view = mode;
        self.apply_runtime_settings();
        self.persist_settings();
    }

    fn command_action_set_theme(&mut self, theme: crate::cli::ThemeArg) {
        self.settings.theme = theme;
        self.apply_runtime_settings();
        self.persist_settings();
    }

    fn push_command_bar_char(&mut self, ch: char, modifiers: KeyModifiers) {
        if modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) {
            return;
        }
        if self.command_bar.buffer.chars().count() >= 100 {
            return;
        }
        self.command_bar.buffer.push(ch);
        self.command_bar.parse_error = None;
    }
}

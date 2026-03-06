use crate::Result;
use crate::app::state::input::settings_close_key;
use crate::app::state::settings::adjust_setting_selection;
use crate::app::state::{AppState, SettingsSelection};
use crate::cli::Cli;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

use crate::app::state::AppEvent;

pub(crate) fn ctrl_char(key: KeyEvent, target: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&target))
}

impl AppState {
    pub(crate) async fn handle_settings_key(
        &mut self,
        code: KeyCode,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        if settings_close_key(code) {
            self.settings_open = false;
            return Ok(());
        }
        if self.handle_settings_nav_key(code) {
            return Ok(());
        }
        if matches!(code, KeyCode::Enter) {
            self.handle_settings_enter(tx, cli).await?;
        }
        Ok(())
    }

    fn handle_settings_nav_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Up => {
                self.settings_selected = self.settings_selected.prev();
                true
            }
            KeyCode::Down => {
                self.settings_selected = self.settings_selected.next();
                true
            }
            KeyCode::Left => {
                self.adjust_selected_setting(-1);
                true
            }
            KeyCode::Right => {
                self.adjust_selected_setting(1);
                true
            }
            _ => false,
        }
    }

    pub(crate) async fn handle_settings_enter(
        &mut self,
        tx: &mpsc::Sender<AppEvent>,
        cli: &Cli,
    ) -> Result<()> {
        match self.settings_selected {
            SettingsSelection::RefreshNow => self.start_fetch(tx, cli).await?,
            SettingsSelection::Close => self.settings_open = false,
            _ => self.adjust_selected_setting(1),
        }
        Ok(())
    }

    pub(crate) async fn handle_help_key(
        &mut self,
        key: KeyEvent,
        tx: &mpsc::Sender<AppEvent>,
    ) -> Result<()> {
        if matches!(key.code, KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?')) {
            self.help_open = false;
            return Ok(());
        }
        if ctrl_char(key, 'c') {
            tx.send(AppEvent::Quit).await?;
            return Ok(());
        }
        if ctrl_char(key, 'l') {
            tx.send(AppEvent::ForceRedraw).await?;
        }
        Ok(())
    }

    pub(crate) fn adjust_selected_setting(&mut self, direction: i8) {
        let changed = adjust_setting_selection(self, self.settings_selected, direction);

        if changed {
            self.apply_runtime_settings();
            self.persist_settings();
        }
    }

    pub(crate) fn apply_runtime_settings(&mut self) {
        let previous_signature = self
            .last_render_signature
            .clone()
            .unwrap_or_else(|| self.render_signature());
        let hero_visual_changed = previous_signature.hero_visual != self.settings.hero_visual;
        let previous_motion_mode = self.motion_mode;
        self.units = self.settings.units;
        self.hourly_view_mode = self.settings.hourly_view;
        self.motion_mode = self.settings.motion_mode;
        self.animate_ui = self.motion_mode.allows_animation();
        if !self.settings.command_bar_enabled {
            self.command_bar.close();
        }
        self.refresh_interval_secs_runtime
            .store(self.settings.refresh_interval_secs, Ordering::Relaxed);
        self.particles
            .set_options(self.motion_mode, self.settings.no_flash);
        if hero_visual_changed {
            self.begin_transition(
                crate::ui::animation::SceneTransitionState::hero_visual_switch(self.motion_mode),
            );
        } else if previous_motion_mode != self.motion_mode && self.motion_mode.allows_transitions()
        {
            self.begin_transition(crate::ui::animation::SceneTransitionState::fetch_reveal(
                self.motion_mode,
            ));
        }
        self.sync_motion_profile();
        self.last_render_signature = Some(self.render_signature());
    }

    pub(crate) fn persist_settings(&mut self) {
        if self.demo_mode {
            return;
        }
        if let Some(path) = &self.settings_path
            && let Err(err) = crate::app::settings::save_runtime_settings(path, &self.settings)
        {
            self.last_error = Some(format!("Failed to save settings: {err}"));
        }
    }
}

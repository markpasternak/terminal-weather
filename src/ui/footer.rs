use std::{borrow::Cow, fmt::Write};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::state::AppState, update::UpdateStatus};

use super::theme::Theme;

const WIDE_ACTIONS: [(&str, &str); 5] = [
    ("R", "Refresh"),
    ("V", "Hourly View"),
    ("L", "Cities"),
    ("S", "Settings"),
    ("<-/->", "Scroll"),
];
const MEDIUM_ACTIONS: [(&str, &str); 5] = [
    ("R", "Refresh"),
    ("V", "View"),
    ("L", "Cities"),
    ("S", "Settings"),
    ("<-/->", "Scroll"),
];
const COMPACT_ACTIONS: [(&str, &str); 4] = [
    ("R", "Refresh"),
    ("V", "View"),
    ("L", "Cities"),
    ("S", "Settings"),
];
const TINY_ACTIONS: [(&str, &str); 1] = [("R", "Refresh")];

pub(super) fn render_bottom_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.command_bar.open {
        render_command_bar(frame, area, state);
    } else {
        render_footer(frame, area, state);
    }
}

pub(crate) fn footer_text_for_width(
    width: u16,
    state: &AppState,
    theme: Theme,
) -> Vec<Span<'static>> {
    let base = base_footer_text_for_width(width, state, theme);
    append_update_hint(width, base, &state.update_status, theme)
}

pub(crate) fn update_hint_for_width(width: u16, status: &UpdateStatus) -> Option<String> {
    let latest = match status {
        UpdateStatus::UpdateAvailable { latest } => latest,
        UpdateStatus::Unknown | UpdateStatus::UpToDate => return None,
    };
    if width >= 110 {
        Some(format!(
            "Update available: v{latest} · brew upgrade markpasternak/tap/terminal-weather"
        ))
    } else if width >= 72 {
        Some(format!("Update available: v{latest}"))
    } else {
        None
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = super::theme::resolved_theme(state);
    let mut text_spans = footer_text_for_width(area.width, state, theme);
    push_footer_action(&mut text_spans, "F1/?", "Help", theme.accent, theme);
    let footer = Paragraph::new(Line::from(text_spans)).style(Style::default().bg(theme.surface));

    frame.render_widget(footer, area);
}

fn render_command_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = super::theme::resolved_theme(state);
    let error_suffix = state
        .command_bar
        .parse_error
        .as_deref()
        .map_or_else(String::new, |err| format!("  ! {err}"));
    let buffer = state.command_bar.buffer.as_str();
    let line = format!("{buffer}{error_suffix}");
    let content = if line.is_empty() {
        ":".to_string()
    } else {
        line
    };
    let widget =
        Paragraph::new(content.clone()).style(Style::default().fg(theme.accent).bg(theme.surface));
    frame.render_widget(widget, area);

    let cursor_x = area.x + Line::from(buffer).width() as u16;
    frame.set_cursor_position((cursor_x, area.y));
}

fn base_footer_text_for_width(width: u16, state: &AppState, theme: Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (key, label) in fixed_footer_actions(width) {
        push_footer_action(&mut spans, key, label, theme.text, theme);
    }
    let quit_label = quit_label(width, state);
    push_footer_action(&mut spans, "Q", quit_label.as_ref(), theme.text, theme);
    spans
}

fn fixed_footer_actions(width: u16) -> &'static [(&'static str, &'static str)] {
    if width >= 92 {
        &WIDE_ACTIONS
    } else if width >= 72 {
        &MEDIUM_ACTIONS
    } else if width >= 52 {
        &COMPACT_ACTIONS
    } else {
        &TINY_ACTIONS
    }
}

fn quit_label(width: u16, state: &AppState) -> Cow<'static, str> {
    let mut label = String::from("Quit");
    if width >= 92 {
        let _ = write!(label, "  Tab Focus({})", state.panel_focus.label());
    }
    if width >= 52 && state.settings.command_bar_enabled {
        label.push_str("  : Command");
    }
    Cow::Owned(label)
}

fn push_footer_action(
    spans: &mut Vec<Span<'static>>,
    key: &str,
    label: &str,
    key_color: ratatui::style::Color,
    theme: Theme,
) {
    spans.push(Span::styled(
        key.to_string(),
        Style::default().fg(key_color).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        format!(" {label}  "),
        Style::default().fg(theme.muted_text),
    ));
}

fn append_update_hint(
    width: u16,
    mut base: Vec<Span<'static>>,
    status: &UpdateStatus,
    theme: Theme,
) -> Vec<Span<'static>> {
    let Some(hint) = update_hint_for_width(width, status) else {
        return base;
    };
    if base.is_empty() {
        base.push(Span::styled(hint, Style::default().fg(theme.muted_text)));
        return base;
    }
    base.push(Span::styled(
        format!("  {hint}"),
        Style::default().fg(theme.muted_text),
    ));
    base
}

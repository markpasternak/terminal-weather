#![allow(clippy::cast_possible_truncation)]

pub mod animation;
pub mod layout;
pub mod narrative;
pub mod particles;
pub mod symbols;
pub mod theme;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::state::{AppMode, AppState},
    cli::Cli,
    domain::weather::{HourlyViewMode, weather_label_for_time},
    resilience::freshness::FreshnessState,
    ui::{
        animation::{SeededMotion, UiMotionContext},
        narrative::build_narrative,
        theme::resolved_theme,
    },
    update::UpdateStatus,
};

const MIN_RENDER_WIDTH: u16 = 20;
const MIN_RENDER_HEIGHT: u16 = 10;

#[must_use]
pub fn motion_context(state: &AppState, lane: &str) -> UiMotionContext {
    let animate = state.animate_ui && state.motion_mode.allows_animation();
    UiMotionContext {
        elapsed_seconds: if animate {
            state.animation_clock.elapsed_seconds
        } else {
            0.0
        },
        dt_seconds: if animate {
            state.animation_clock.dt_seconds
        } else {
            0.0
        },
        frame_index: state.animation_clock.frame_index,
        motion_mode: state.motion_mode,
        seed: SeededMotion::new(state.motion_seed(lane)),
        weather_profile: state.weather_motion_profile,
        transition_progress: state.transition_progress(),
        animate,
    }
}

pub fn render(frame: &mut Frame, state: &AppState, cli: &Cli) {
    let area = frame.area();
    let theme = resolved_theme(state);

    if area.width < MIN_RENDER_WIDTH || area.height < MIN_RENDER_HEIGHT {
        render_small_terminal_hint(frame, area, theme, state);
        return;
    }

    let content_area = content_area_with_footer(frame, area, state);
    let alerts = state
        .weather
        .as_ref()
        .map(|bundle| crate::domain::alerts::scan_alerts(bundle, state.units))
        .unwrap_or_default();
    render_main_panels(frame, content_area, state, cli, &alerts);
    render_status_badge(frame, content_area, state);
    render_modal_overlay(frame, area, state, cli);
}

fn render_small_terminal_hint(
    frame: &mut Frame,
    area: Rect,
    theme: crate::ui::theme::Theme,
    state: &AppState,
) {
    let mut lines: Vec<Line> = compact_logo_lines(area.width.saturating_sub(2))
        .into_iter()
        .map(|line| {
            Line::from(line).style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    lines.push(Line::from(""));
    if let Some(weather) = &state.weather {
        lines.push(Line::from(format!(
            "{}° {}",
            weather.current_temp(state.units),
            weather_label_for_time(weather.current.weather_code, weather.current.is_day)
        )));
        let narrative = build_narrative(state, weather);
        lines.push(Line::from(narrative.now_action));
        lines.push(Line::from(format!(
            "{} Confidence {} · {}",
            narrative.confidence_symbol,
            narrative.confidence.label(),
            narrative.reliability
        )));
    }
    lines.push(Line::from("Too small for full render"));
    lines.push(Line::from(format!(
        "Need {MIN_RENDER_WIDTH}x{MIN_RENDER_HEIGHT}+"
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Tip: press ", Style::default().fg(theme.muted_text)),
        Span::styled(
            "Q",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit", Style::default().fg(theme.muted_text)),
    ]));

    let warning = Paragraph::new(lines)
        .style(Style::default().fg(theme.text).bg(theme.surface))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("terminal weather")
                .style(Style::default().fg(theme.text).bg(theme.surface))
                .border_style(Style::default().fg(theme.border).bg(theme.surface)),
        );
    frame.render_widget(warning, area);
}

fn content_area_with_footer(frame: &mut Frame, area: Rect, state: &AppState) -> Rect {
    let overlays_open = has_modal_overlay(state);
    let show_footer = !overlays_open && area.height > MIN_RENDER_HEIGHT;
    if !show_footer {
        return area;
    }
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    render_bottom_bar(frame, sections[1], state);
    sections[0]
}

fn render_main_panels(
    frame: &mut Frame,
    content_area: Rect,
    state: &AppState,
    cli: &Cli,
    alerts: &[crate::domain::alerts::WeatherAlert],
) {
    let constraints = panel_constraints(content_area, state.hourly_view_mode);
    let alert_height = crate::ui::widgets::alerts::alert_row_height(alerts);

    if alert_height > 0 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                constraints[0],
                Constraint::Length(alert_height),
                constraints[1],
                constraints[2],
            ])
            .split(content_area);

        widgets::hero::render(frame, chunks[0], state, cli);
        widgets::alerts::render(frame, chunks[1], alerts, state);
        widgets::hourly::render(frame, chunks[2], state, cli);
        widgets::daily::render(frame, chunks[3], state, cli);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);
    widgets::hero::render(frame, chunks[0], state, cli);
    widgets::hourly::render(frame, chunks[1], state, cli);
    widgets::daily::render(frame, chunks[2], state, cli);
}

fn render_modal_overlay(frame: &mut Frame, area: Rect, state: &AppState, cli: &Cli) {
    if state.mode == AppMode::SelectingLocation {
        widgets::selector::render(frame, centered_rect(70, 60, area), state);
    } else if state.settings_open {
        widgets::settings::render(frame, centered_rect(68, 74, area), state);
    } else if state.city_picker_open {
        widgets::city_picker::render(frame, centered_rect(74, 74, area), state);
    } else if state.help_open {
        widgets::help::render(frame, centered_rect(82, 84, area), state, cli);
    }
}

fn compact_logo_lines(inner_width: u16) -> Vec<&'static str> {
    if inner_width >= 16 {
        vec![
            "terminal-weather",
            " .--.   .--.   ",
            " (tw)---(wx)   ",
            " '--'   '--'   ",
        ]
    } else if inner_width >= 8 {
        vec!["terminal", "weather", "[ tw ]"]
    } else {
        vec!["tw"]
    }
}

fn render_status_badge(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = resolved_theme(state);

    let label = match state.refresh_meta.state {
        FreshnessState::Offline => Some((freshness_badge_text("⚠ offline", state), theme.danger)),
        FreshnessState::Stale => Some((freshness_badge_text("⚠ stale", state), theme.warning)),
        FreshnessState::Fresh if state.fetch_in_flight => {
            Some((format!("{} syncing", spinner(state.frame_tick)), theme.info))
        }
        FreshnessState::Fresh => None,
    };

    if let Some((text, color)) = label {
        let width = (text.chars().count() as u16 + 2).min(area.width);
        let badge_area = Rect {
            x: area.right().saturating_sub(width + 1),
            y: area.y,
            width,
            height: 1,
        };
        let badge = Paragraph::new(Line::from(text))
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD));
        frame.render_widget(badge, badge_area);
    }
}

fn freshness_badge_text(label: &str, state: &AppState) -> String {
    state.refresh_meta.retry_in_seconds().map_or_else(
        || label.to_string(),
        |secs| format!("{label} · retry {secs}s"),
    )
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let theme = resolved_theme(state);
    let mut text_spans = footer_text_for_width(area.width, state, theme);
    text_spans.push(Span::raw("  "));
    text_spans.push(Span::styled("F1/? Help", Style::default().fg(theme.accent)));
    let footer = Paragraph::new(Line::from(text_spans)).style(Style::default().bg(theme.surface));

    frame.render_widget(footer, area);
}

fn render_bottom_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.command_bar.open {
        render_command_bar(frame, area, state);
    } else {
        render_footer(frame, area, state);
    }
}

fn render_command_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = resolved_theme(state);
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

fn footer_text_for_width(
    width: u16,
    state: &AppState,
    theme: crate::ui::theme::Theme,
) -> Vec<Span<'static>> {
    let base = base_footer_text_for_width(width, state, theme);
    append_update_hint(width, base, &state.update_status, theme)
}

fn base_footer_text_for_width(
    width: u16,
    state: &AppState,
    theme: crate::ui::theme::Theme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let command_hint = if state.settings.command_bar_enabled {
        "  : Command".to_string()
    } else {
        String::new()
    };
    let focus_hint = format!("  Tab Focus({})", state.panel_focus.label());

    if width >= 92 {
        spans.extend(vec![
            Span::styled(
                "R",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Refresh  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "V",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Hourly View  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "L",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cities  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "S",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Settings  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "<-/->",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Scroll  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "Q",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" Quit{focus_hint}{command_hint}"),
                Style::default().fg(theme.muted_text),
            ),
        ]);
    } else if width >= 72 {
        spans.extend(vec![
            Span::styled(
                "R",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Refresh  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "V",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" View  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "L",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cities  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "S",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Settings  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "<-/->",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Scroll  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "Q",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" Quit{command_hint}"),
                Style::default().fg(theme.muted_text),
            ),
        ]);
    } else if width >= 52 {
        spans.extend(vec![
            Span::styled(
                "R",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Refresh  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "V",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" View  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "L",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cities  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "S",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Settings  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "Q",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" Quit{command_hint}"),
                Style::default().fg(theme.muted_text),
            ),
        ]);
    } else {
        spans.extend(vec![
            Span::styled(
                "R",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Refresh  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                "Q",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit", Style::default().fg(theme.muted_text)),
        ]);
    }

    spans
}

fn append_update_hint(
    width: u16,
    mut base: Vec<Span<'static>>,
    status: &UpdateStatus,
    theme: crate::ui::theme::Theme,
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

fn update_hint_for_width(width: u16, status: &UpdateStatus) -> Option<String> {
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

fn panel_constraints(content_area: Rect, requested_hourly_mode: HourlyViewMode) -> [Constraint; 3] {
    let use_table_layout =
        requested_hourly_mode == HourlyViewMode::Table || content_area.width < 70;

    if use_table_layout {
        return table_constraints(content_area.height);
    }

    if let Some(hourly_len) = adaptive_hourly_length(content_area.height, requested_hourly_mode) {
        let hero_min = if requested_hourly_mode == HourlyViewMode::Hybrid {
            7
        } else {
            6
        };
        return [
            Constraint::Min(hero_min),
            Constraint::Length(hourly_len),
            Constraint::Min(6),
        ];
    }

    default_constraints()
}

fn table_constraints(height: u16) -> [Constraint; 3] {
    if height >= 60 {
        [
            Constraint::Percentage(52),
            Constraint::Percentage(18),
            Constraint::Percentage(30),
        ]
    } else if height >= 52 {
        [
            Constraint::Percentage(54),
            Constraint::Percentage(18),
            Constraint::Percentage(28),
        ]
    } else if height >= 40 {
        [
            Constraint::Percentage(50),
            Constraint::Percentage(18),
            Constraint::Percentage(32),
        ]
    } else if height >= 32 {
        [
            Constraint::Percentage(46),
            Constraint::Percentage(20),
            Constraint::Percentage(34),
        ]
    } else {
        default_constraints()
    }
}

fn adaptive_hourly_length(height: u16, mode: HourlyViewMode) -> Option<u16> {
    if height < 22 {
        return None;
    }

    match mode {
        HourlyViewMode::Hybrid => Some(hybrid_hourly_length(height)),
        HourlyViewMode::Chart => Some(chart_hourly_length(height)),
        HourlyViewMode::Table => None,
    }
}

fn hybrid_hourly_length(height: u16) -> u16 {
    if height >= 36 {
        11
    } else if height >= 28 {
        10
    } else {
        9
    }
}

fn chart_hourly_length(height: u16) -> u16 {
    if height >= 36 {
        13
    } else if height >= 28 {
        11
    } else {
        10
    }
}

fn default_constraints() -> [Constraint; 3] {
    [
        Constraint::Percentage(42),
        Constraint::Percentage(22),
        Constraint::Percentage(36),
    ]
}

fn has_modal_overlay(state: &AppState) -> bool {
    state.mode == AppMode::SelectingLocation
        || state.settings_open
        || state.city_picker_open
        || state.help_open
}

fn spinner(frame_tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[(frame_tick as usize) % FRAMES.len()]
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests;

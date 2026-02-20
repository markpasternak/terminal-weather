#![allow(clippy::cast_possible_truncation)]

pub mod layout;
pub mod particles;
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
    domain::weather::{HourlyViewMode, WeatherCategory, weather_code_to_category},
    resilience::freshness::FreshnessState,
    ui::theme::{detect_color_capability, theme_for},
};

const MIN_RENDER_WIDTH: u16 = 20;
const MIN_RENDER_HEIGHT: u16 = 10;

pub fn render(frame: &mut Frame, state: &AppState, cli: &Cli) {
    let area = frame.area();
    let theme = theme_for_state(state);

    if area.width < MIN_RENDER_WIDTH || area.height < MIN_RENDER_HEIGHT {
        render_small_terminal_hint(frame, area, theme);
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

fn theme_for_state(state: &AppState) -> crate::ui::theme::Theme {
    let capability = detect_color_capability(state.color_mode);
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    theme_for(category, is_day, capability, state.settings.theme)
}

fn render_small_terminal_hint(frame: &mut Frame, area: Rect, theme: crate::ui::theme::Theme) {
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
    lines.push(Line::from("Too small for full render"));
    lines.push(Line::from(format!(
        "Need {MIN_RENDER_WIDTH}x{MIN_RENDER_HEIGHT}+"
    )));

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
    render_footer(frame, sections[1], state);
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
    let capability = detect_color_capability(state.color_mode);
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    let theme = theme_for(category, is_day, capability, state.settings.theme);

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

    let capability = detect_color_capability(state.color_mode);
    let (category, is_day) =
        state
            .weather
            .as_ref()
            .map_or((WeatherCategory::Unknown, false), |w| {
                (
                    weather_code_to_category(w.current.weather_code),
                    w.current.is_day,
                )
            });
    let theme = theme_for(category, is_day, capability, state.settings.theme);
    let text = footer_text_for_width(area.width);
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(text, Style::default().fg(theme.muted_text)),
        Span::raw("  "),
        Span::styled("F1/? Help", Style::default().fg(theme.accent)),
    ]))
    .style(Style::default().bg(theme.surface));

    frame.render_widget(footer, area);
}

fn footer_text_for_width(width: u16) -> &'static str {
    if width >= 92 {
        "r Refresh  v Hourly View  l Cities  s Settings  <-/-> Scroll  q Quit"
    } else if width >= 72 {
        "r Refresh  v View  l Cities  s Settings  <-/-> Scroll  q Quit"
    } else if width >= 52 {
        "r Refresh  v View  l Cities  s Settings  q Quit"
    } else {
        "r Refresh  q Quit"
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

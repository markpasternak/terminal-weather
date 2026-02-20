pub mod background;
pub mod landmark_panel;
pub mod weather;
pub mod weather_expanded;

use crate::{
    app::state::AppState,
    cli::Cli,
    domain::weather::{WeatherCategory, weather_code_to_category},
    ui::theme::{detect_color_capability, theme_for},
};
use background::GradientBackground;
use landmark_panel::render_landmark;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders},
};
use weather::render_weather_info;

fn inset_rect(r: Rect, dw: u16, dh: u16) -> Rect {
    let w = r.width.saturating_sub(dw * 2);
    let h = r.height.saturating_sub(dh * 2);
    Rect {
        x: r.x.saturating_add(dw),
        y: r.y.saturating_add(dh),
        width: w,
        height: h,
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let (is_day, code, theme) = hero_context(state);
    render_hero_background(frame, area, state, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Current · L cities · S settings · ? help")
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = split_hero_columns(inner);
    let left_area = left_weather_area(columns[0]);
    render_weather_info(frame, left_area, state, theme, code);

    if columns.len() > 1 {
        render_right_landmark(frame, columns[1], state, is_day, theme);
    }
}

fn hero_context(state: &AppState) -> (bool, u8, crate::ui::theme::Theme) {
    let (category, is_day, code) = state.weather.as_ref().map_or(
        // Loading/no-data should default to a dark palette to avoid bright blank panels.
        (WeatherCategory::Unknown, false, 0),
        |w| {
            (
                weather_code_to_category(w.current.weather_code),
                w.current.is_day,
                w.current.weather_code,
            )
        },
    );
    let capability = detect_color_capability(state.color_mode);
    let theme = theme_for(category, is_day, capability, state.settings.theme);
    (is_day, code, theme)
}

fn render_hero_background(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
) {
    let bg = GradientBackground {
        top: theme.top,
        bottom: theme.bottom,
        text: theme.text,
        particle: theme.particle,
        particles: &state.particles.particles,
        flash: state.particles.flash_active(),
        flash_bg: theme.accent,
        flash_fg: theme.text,
    };
    frame.render_widget(bg, area);
}

fn split_hero_columns(inner: Rect) -> Vec<Rect> {
    if inner.width < 58 || inner.height < 8 {
        return vec![inner];
    }

    let (left_pct, right_pct) = if inner.width >= 170 {
        (38, 62)
    } else if inner.width >= 140 {
        (40, 60)
    } else if inner.width >= 120 {
        (42, 58)
    } else if inner.width >= 96 {
        (48, 52)
    } else {
        (58, 42)
    };
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(inner)
        .to_vec()
}

fn left_weather_area(column: Rect) -> Rect {
    if column.width >= 56 {
        inset_rect(column, 1, 0)
    } else {
        column
    }
}

fn render_right_landmark(
    frame: &mut Frame,
    right_column: Rect,
    state: &AppState,
    is_day: bool,
    theme: crate::ui::theme::Theme,
) {
    let separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme.border));
    let right_inner = separator.inner(right_column);
    frame.render_widget(separator, right_column);

    let right_content = if right_inner.width >= 78 && right_inner.height >= 14 {
        inset_rect(right_inner, 3, 1)
    } else if right_inner.width >= 60 && right_inner.height >= 10 {
        inset_rect(right_inner, 2, 1)
    } else if right_inner.width >= 48 {
        inset_rect(right_inner, 2, 0)
    } else {
        right_inner
    };
    render_landmark(frame, right_content, state, is_day, theme);
}

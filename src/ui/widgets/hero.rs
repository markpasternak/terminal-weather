use chrono::Local;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    app::state::{AppMode, AppState},
    cli::Cli,
    domain::weather::{WeatherCategory, weather_code_to_category, weather_label},
    ui::theme::{detect_color_capability, theme_for},
    ui::widgets::landmark::{LandmarkTint, scene_for_location},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, _cli: &Cli) {
    let (category, is_day, code) = state
        .weather
        .as_ref()
        .map(|w| {
            (
                weather_code_to_category(w.current.weather_code),
                w.current.is_day,
                w.current.weather_code,
            )
        })
        .unwrap_or((WeatherCategory::Unknown, true, 0));

    let capability = detect_color_capability();
    let theme = theme_for(category, is_day, capability);

    let bg = GradientBackground {
        top: theme.top,
        bottom: theme.bottom,
        text: theme.text,
        particle: theme.particle,
        particles: &state.particles.particles,
        flash: state.particles.flash_active(),
    };
    frame.render_widget(bg, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Current")
        .border_style(Style::default().fg(theme.text));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = if inner.width >= 58 && inner.height >= 8 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(inner)
            .to_vec()
    } else {
        vec![inner]
    };

    render_weather_info(frame, columns[0], state, theme.text, theme.muted_text, code);

    if columns.len() > 1 {
        let right = columns[1];
        let separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.muted_text));
        let right_inner = separator.inner(right);
        frame.render_widget(separator, right);
        render_landmark(frame, right_inner, state, is_day, theme);
    }
}

fn render_weather_info(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    text_color: Color,
    muted_color: Color,
    code: u8,
) {
    let mut lines = Vec::new();

    if let Some(weather) = &state.weather {
        let temp = weather.current_temp(state.units);
        let unit_symbol = if state.units == crate::domain::weather::Units::Celsius {
            "C"
        } else {
            "F"
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{temp}°{unit_symbol}"),
            Style::default().fg(text_color).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(Span::styled(
            weather_label(code),
            Style::default().fg(text_color),
        )));
        if let Some((high, low)) = weather.high_low(state.units) {
            lines.push(Line::from(Span::styled(
                format!("H:{high}°  L:{low}°"),
                Style::default().fg(text_color),
            )));
        }
        lines.push(Line::from(Span::styled(
            weather.location.display_name(),
            Style::default().fg(text_color),
        )));

        let freshness = match state.refresh_meta.state {
            crate::resilience::freshness::FreshnessState::Fresh => None,
            crate::resilience::freshness::FreshnessState::Stale => Some("⚠ stale"),
            crate::resilience::freshness::FreshnessState::Offline => Some("⚠ offline"),
        };

        if let Some(flag) = freshness {
            lines.push(Line::from(Span::styled(
                flag,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let updated = state
            .refresh_meta
            .last_success
            .map(|ts| {
                let local = ts.with_timezone(&Local);
                let mins = state.refresh_meta.age_minutes().unwrap_or(0);
                format!(
                    "Last updated: {} ({}m ago)",
                    local.format("%H:%M"),
                    mins.max(0)
                )
            })
            .unwrap_or_else(|| "Last updated: --:--".to_string());
        lines.push(Line::from(Span::styled(
            updated,
            Style::default().fg(muted_color),
        )));
    } else if state.mode == AppMode::Error {
        lines.push(Line::from(Span::styled(
            "Unable to load weather",
            Style::default().fg(text_color),
        )));
        if let Some(err) = &state.last_error {
            lines.push(Line::from(Span::styled(
                err.clone(),
                Style::default().fg(muted_color),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            state.loading_message.clone(),
            Style::default().fg(text_color),
        )));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_landmark(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    is_day: bool,
    theme: crate::ui::theme::Theme,
) {
    if area.width < 10 || area.height < 4 {
        return;
    }

    let location_name = state
        .weather
        .as_ref()
        .map(|w| w.location.name.as_str())
        .unwrap_or("Local");

    let scene = scene_for_location(
        location_name,
        is_day,
        state.frame_tick,
        state.animate_ui,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );

    let tint = match scene.tint {
        LandmarkTint::Warm => Color::LightYellow,
        LandmarkTint::Cool => Color::LightCyan,
        LandmarkTint::Neutral => theme.muted_text,
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} {}",
            if state.animate_ui { "~>" } else { "--" },
            scene.label
        ),
        Style::default()
            .fg(theme.muted_text)
            .add_modifier(Modifier::BOLD),
    )));

    for line in scene.lines {
        lines.push(Line::from(Span::styled(line, Style::default().fg(tint))));
    }

    let mut text = Text::from(lines);
    text = text.patch_style(Style::default().fg(tint));
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

struct GradientBackground<'a> {
    top: Color,
    bottom: Color,
    text: Color,
    particle: Color,
    particles: &'a [crate::ui::particles::Particle],
    flash: bool,
}

impl Widget for GradientBackground<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for y in area.top()..area.bottom() {
            let t = if area.height <= 1 {
                0.0
            } else {
                (y - area.y) as f32 / (area.height - 1) as f32
            };
            let color = lerp_color(self.top, self.bottom, t);
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("cell");
                cell.set_symbol(" ").set_bg(color).set_fg(self.text);
            }
        }

        if self.flash {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    let cell = buf.cell_mut((x, y)).expect("cell");
                    cell.set_bg(Color::White).set_fg(Color::Black);
                }
            }
        }

        for p in self.particles {
            let x = area.x + ((p.x.clamp(0.0, 1.0)) * area.width as f32) as u16;
            let y = area.y + ((p.y.clamp(0.0, 1.0)) * area.height as f32) as u16;
            if x >= area.right() || y >= area.bottom() {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                let bg = cell.bg;
                cell.set_symbol(&p.glyph.to_string())
                    .set_fg(self.particle)
                    .set_bg(bg);
            }
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => {
            let lerp = |x: u8, y: u8| -> u8 {
                (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            Color::Rgb(lerp(ar, br), lerp(ag, bg), lerp(ab, bb))
        }
        _ => a,
    }
}

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
    cli::{Cli, SilhouetteSourceArg},
    domain::weather::{
        WeatherCategory, convert_temp, round_temp, weather_code_to_category, weather_label,
    },
    ui::theme::{condition_color, detect_color_capability, theme_for},
    ui::widgets::landmark::{LandmarkTint, scene_for_location, scene_from_web_art},
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
        // Loading/no-data should default to a dark palette to avoid bright blank panels.
        .unwrap_or((WeatherCategory::Unknown, false, 0));

    let capability = detect_color_capability();
    let theme = theme_for(category, is_day, capability, state.settings.theme);

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
        .title("Current · Press S for settings")
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = if inner.width >= 58 && inner.height >= 8 {
        let (left_pct, right_pct) = if inner.width >= 120 {
            (44, 56)
        } else if inner.width >= 96 {
            (50, 50)
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
    } else {
        vec![inner]
    };

    render_weather_info(frame, columns[0], state, theme, code);

    if columns.len() > 1 {
        let right = columns[1];
        let separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.border));
        let right_inner = separator.inner(right);
        frame.render_widget(separator, right);
        render_landmark(frame, right_inner, state, is_day, theme);
    }
}

fn render_weather_info(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
    code: u8,
) {
    let mut lines = Vec::new();
    let compact_metrics = area.width < 54 || area.height < 8;
    let text_color = theme.text;
    let muted_color = theme.muted_text;

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
            Style::default()
                .fg(condition_color(&theme, weather_code_to_category(code)))
                .add_modifier(Modifier::BOLD),
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

        let feels = round_temp(convert_temp(
            weather.current.apparent_temperature_c,
            state.units,
        ));
        let humidity = weather.current.relative_humidity_2m.round() as i32;
        let wind_dir = compass(weather.current.wind_direction_10m);
        let wind = weather.current.wind_speed_10m.round() as i32;
        let precip_now = weather
            .hourly
            .first()
            .and_then(|h| h.precipitation_probability)
            .map(|v| format!("{}%", v.round() as i32))
            .unwrap_or_else(|| "--".to_string());
        let uv_today = weather
            .daily
            .first()
            .and_then(|d| d.uv_index_max)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "--".to_string());
        let cloud = cloud_descriptor(code);

        if compact_metrics {
            lines.push(Line::from(vec![
                Span::styled("Wind ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{wind} km/h {wind_dir}"),
                    Style::default().fg(theme.success),
                ),
                Span::raw("  "),
                Span::styled("Hum ", Style::default().fg(muted_color)),
                Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Feels ", Style::default().fg(muted_color)),
                Span::styled(format!("{feels}°"), Style::default().fg(text_color)),
                Span::raw("  "),
                Span::styled("Humidity ", Style::default().fg(muted_color)),
                Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Wind ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{wind} km/h {wind_dir}"),
                    Style::default().fg(theme.success),
                ),
                Span::raw("  "),
                Span::styled("Precip ", Style::default().fg(muted_color)),
                Span::styled(precip_now, Style::default().fg(theme.accent)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Clouds ", Style::default().fg(muted_color)),
                Span::styled(cloud, Style::default().fg(theme.landmark_neutral)),
                Span::raw("  "),
                Span::styled("UV ", Style::default().fg(muted_color)),
                Span::styled(uv_today, Style::default().fg(theme.warning)),
            ]));
        }

        let freshness = match state.refresh_meta.state {
            crate::resilience::freshness::FreshnessState::Fresh => None,
            crate::resilience::freshness::FreshnessState::Stale => Some("⚠ stale"),
            crate::resilience::freshness::FreshnessState::Offline => Some("⚠ offline"),
        };

        if let Some(flag) = freshness {
            lines.push(Line::from(Span::styled(
                flag,
                Style::default()
                    .fg(match state.refresh_meta.state {
                        crate::resilience::freshness::FreshnessState::Offline => theme.danger,
                        _ => theme.warning,
                    })
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
        let spinner = loading_spinner(state.frame_tick);
        let stage = loading_stage(state.frame_tick);
        let bar = indeterminate_bar(state.frame_tick, 18);

        lines.push(Line::from(Span::styled(
            format!("{spinner} Preparing atmosphere"),
            Style::default().fg(text_color).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            stage,
            Style::default().fg(text_color),
        )));
        lines.push(Line::from(Span::styled(
            bar,
            Style::default().fg(muted_color),
        )));
        lines.push(Line::from(Span::styled(
            state.loading_message.clone(),
            Style::default().fg(text_color),
        )));
        lines.push(Line::from(Span::styled(
            "Tip: press s for settings, r to retry, q to quit",
            Style::default().fg(muted_color),
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
        .or_else(|| state.selected_location.as_ref().map(|l| l.name.as_str()))
        .unwrap_or("Local");

    let scene = if matches!(
        state.settings.silhouette_source,
        SilhouetteSourceArg::Web | SilhouetteSourceArg::Auto
    ) {
        state
            .active_location_key
            .as_ref()
            .and_then(|key| state.web_silhouettes.get(key))
            .map(|art| {
                scene_from_web_art(
                    art,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            })
            .unwrap_or_else(|| {
                if matches!(state.settings.silhouette_source, SilhouetteSourceArg::Web)
                    && state
                        .active_location_key
                        .as_ref()
                        .is_some_and(|key| state.silhouettes_in_flight.contains(key))
                {
                    scene_from_web_art(
                        &crate::domain::weather::SilhouetteArt {
                            label: "Fetching web silhouette".to_string(),
                            lines: vec![
                                "  searching landmark image...".to_string(),
                                "  converting image to ascii...".to_string(),
                            ],
                        },
                        area.width.saturating_sub(2),
                        area.height.saturating_sub(2),
                    )
                } else {
                    scene_for_location(
                        location_name,
                        is_day,
                        state.frame_tick,
                        state.animate_ui,
                        area.width.saturating_sub(2),
                        area.height.saturating_sub(2),
                    )
                }
            })
    } else {
        scene_for_location(
            location_name,
            is_day,
            state.frame_tick,
            state.animate_ui,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    };

    let tint = match scene.tint {
        LandmarkTint::Warm => theme.landmark_warm,
        LandmarkTint::Cool => theme.landmark_cool,
        LandmarkTint::Neutral => theme.landmark_neutral,
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} {}",
            if state.animate_ui { "~>" } else { "--" },
            scene.label
        ),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )));

    for line in scene.lines {
        lines.push(Line::from(Span::styled(line, Style::default().fg(tint))));
    }

    let mut text = Text::from(lines);
    text = text.patch_style(Style::default().fg(tint));
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn loading_spinner(frame_tick: u64) -> &'static str {
    const FRAMES: [&str; 8] = ["-", "\\", "|", "/", "-", "\\", "|", "/"];
    FRAMES[(frame_tick as usize) % FRAMES.len()]
}

fn loading_stage(frame_tick: u64) -> &'static str {
    const STAGES: [&str; 3] = [
        "Locating city context",
        "Fetching weather layers",
        "Composing terminal scene",
    ];
    STAGES[((frame_tick / 14) as usize) % STAGES.len()]
}

fn indeterminate_bar(frame_tick: u64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut chars = vec!['·'; width];
    let head = (frame_tick as usize) % width;
    chars[head] = '█';
    if head > 0 {
        chars[head - 1] = '▓';
    }
    if head + 1 < width {
        chars[head + 1] = '▓';
    }
    format!("[{}]", chars.into_iter().collect::<String>())
}

fn compass(deg: f32) -> &'static str {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let mut idx = ((deg.rem_euclid(360.0) / 45.0).round() as usize) % 8;
    if idx >= DIRS.len() {
        idx = 0;
    }
    DIRS[idx]
}

fn cloud_descriptor(code: u8) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mostly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Foggy",
        51..=67 | 80..=82 => "Rain clouds",
        71..=86 => "Snow clouds",
        95 | 96 | 99 => "Storm clouds",
        _ => "Variable",
    }
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

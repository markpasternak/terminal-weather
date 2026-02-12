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
        ColoredGlyph, WeatherCategory, convert_temp, round_temp, weather_code_to_category,
        weather_icon, weather_label,
    },
    ui::theme::{ColorCapability, condition_color, detect_color_capability, quantize, theme_for},
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
        flash_bg: theme.accent,
        flash_fg: theme.text,
    };
    frame.render_widget(bg, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Current · L cities · S settings")
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

    let left_area = if columns[0].width >= 56 {
        inset_rect(columns[0], 1, 0)
    } else {
        columns[0]
    };
    render_weather_info(frame, left_area, state, theme, code);

    if columns.len() > 1 {
        let right = columns[1];
        let separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.border));
        let right_inner = separator.inner(right);
        frame.render_widget(separator, right);
        let right_content = if right_inner.width >= 48 {
            inset_rect(right_inner, 2, 0)
        } else {
            right_inner
        };
        render_landmark(frame, right_content, state, is_day, theme, capability);
    }
}

fn render_weather_info(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
    code: u8,
) {
    let scale = HeroScale::for_area(area);
    let mut lines = Vec::new();
    let compact_metrics = scale.compact_metrics();
    let text_color = theme.text;
    let muted_color = theme.muted_text;

    if let Some(weather) = &state.weather {
        if area.height >= 13 && area.width >= 48 {
            render_weather_info_expanded(frame, area, state, theme, weather, code);
            return;
        }

        let temp = weather.current_temp(state.units);
        let unit_symbol = if state.units == crate::domain::weather::Units::Celsius {
            "C"
        } else {
            "F"
        };
        let metric_gap = scale.metric_gap();
        if matches!(scale, HeroScale::Deluxe) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{temp}°{unit_symbol}"),
                    Style::default().fg(text_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  ·  "),
                Span::styled(
                    weather_label(code),
                    Style::default()
                        .fg(condition_color(&theme, weather_code_to_category(code)))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
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
        }
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
                Span::raw(metric_gap),
                Span::styled("Hum ", Style::default().fg(muted_color)),
                Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Feels ", Style::default().fg(muted_color)),
                Span::styled(format!("{feels}°"), Style::default().fg(text_color)),
                Span::raw(metric_gap),
                Span::styled("Humidity ", Style::default().fg(muted_color)),
                Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Wind ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{wind} km/h {wind_dir}"),
                    Style::default().fg(theme.success),
                ),
                Span::raw(metric_gap),
                Span::styled("Precip ", Style::default().fg(muted_color)),
                Span::styled(precip_now, Style::default().fg(theme.accent)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Clouds ", Style::default().fg(muted_color)),
                Span::styled(cloud, Style::default().fg(theme.landmark_neutral)),
                Span::raw(metric_gap),
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
        render_loading_choreography(frame, area, state, theme, scale);
        return;
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_weather_info_expanded(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
    weather: &crate::domain::weather::ForecastBundle,
    code: u8,
) {
    let scale = HeroScale::for_area(area);
    let sections = if area.height >= 20 {
        Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(4),
        ])
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(3),
        ])
        .split(area)
    };
    let top_area = sections[0];
    let metrics_area = sections[1];
    let trend_area = sections[2];

    let temp = weather.current_temp(state.units);
    let unit_symbol = if state.units == crate::domain::weather::Units::Celsius {
        "C"
    } else {
        "F"
    };
    let condition = weather_label(code);
    let location = weather.location.display_name();
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

    let sunrise = weather
        .daily
        .first()
        .and_then(|d| d.sunrise)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());
    let sunset = weather
        .daily
        .first()
        .and_then(|d| d.sunset)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());

    let updated = state
        .refresh_meta
        .last_success
        .map(|ts| {
            let local = ts.with_timezone(&Local);
            let mins = state.refresh_meta.age_minutes().unwrap_or(0);
            format!(
                "Last updated {} ({}m ago)",
                local.format("%H:%M"),
                mins.max(0)
            )
        })
        .unwrap_or_else(|| "Last updated --:--".to_string());

    let freshness = match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => "Fresh",
        crate::resilience::freshness::FreshnessState::Stale => "⚠ Stale",
        crate::resilience::freshness::FreshnessState::Offline => "⚠ Offline",
    };
    let freshness_color = match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => theme.success,
        crate::resilience::freshness::FreshnessState::Stale => theme.warning,
        crate::resilience::freshness::FreshnessState::Offline => theme.danger,
    };

    let mut top_lines = vec![Line::from(vec![
        Span::styled(
            format!("{temp}°{unit_symbol}  "),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            condition,
            Style::default()
                .fg(condition_color(&theme, weather_code_to_category(code)))
                .add_modifier(Modifier::BOLD),
        ),
    ])];
    if let Some((high, low)) = weather.high_low(state.units) {
        top_lines.push(Line::from(vec![
            Span::styled(
                format!("H:{high}°  L:{low}°  "),
                Style::default().fg(theme.text),
            ),
            Span::styled(location, Style::default().fg(theme.muted_text)),
        ]));
    } else {
        top_lines.push(Line::from(Span::styled(
            location,
            Style::default().fg(theme.muted_text),
        )));
    }
    top_lines.push(Line::from(vec![
        Span::styled("Status ", Style::default().fg(theme.muted_text)),
        Span::styled(
            freshness,
            Style::default()
                .fg(freshness_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    top_lines.push(Line::from(Span::styled(
        updated,
        Style::default().fg(theme.muted_text),
    )));
    frame.render_widget(Paragraph::new(top_lines), top_area);

    let metric_cols = Layout::horizontal(if matches!(scale, HeroScale::Deluxe) {
        [Constraint::Percentage(46), Constraint::Percentage(54)]
    } else {
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    })
    .split(metrics_area);
    let left_metrics = vec![
        Line::from(vec![
            Span::styled("Feels ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{feels}°"), Style::default().fg(theme.text)),
            Span::raw("  "),
            Span::styled("Humidity ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
        ]),
        Line::from(vec![
            Span::styled("Wind ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{wind} km/h {wind_dir}"),
                Style::default().fg(theme.success),
            ),
            Span::raw("  "),
            Span::styled("Precip ", Style::default().fg(theme.muted_text)),
            Span::styled(precip_now, Style::default().fg(theme.accent)),
        ]),
    ];
    frame.render_widget(Paragraph::new(left_metrics), metric_cols[0]);

    let right_metrics = vec![
        Line::from(vec![
            Span::styled("Clouds ", Style::default().fg(theme.muted_text)),
            Span::styled(cloud, Style::default().fg(theme.landmark_neutral)),
            Span::raw("  "),
            Span::styled("UV ", Style::default().fg(theme.muted_text)),
            Span::styled(uv_today, Style::default().fg(theme.warning)),
        ]),
        Line::from(vec![
            Span::styled("Sunrise ", Style::default().fg(theme.muted_text)),
            Span::styled(sunrise, Style::default().fg(theme.warning)),
            Span::raw("  "),
            Span::styled("Sunset ", Style::default().fg(theme.muted_text)),
            Span::styled(sunset, Style::default().fg(theme.warning)),
        ]),
    ];
    frame.render_widget(Paragraph::new(right_metrics), metric_cols[1]);

    let chart_width = trend_area
        .width
        .saturating_sub(scale.chart_left_padding())
        .clamp(12, scale.chart_max_width()) as usize;
    let temp_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.temperature_2m_c.map(|v| convert_temp(v, state.units)))
        .collect::<Vec<_>>();
    let precip_values = weather
        .hourly
        .iter()
        .take(24)
        .map(|h| h.precipitation_probability.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let hours_preview = weather
        .hourly
        .iter()
        .take(((trend_area.width / 9).clamp(4, 10)) as usize)
        .map(|h| {
            format!(
                "{}{}",
                h.time.format("%H"),
                weather_icon(h.weather_code.unwrap_or(code), state.settings.icon_mode)
            )
        })
        .collect::<Vec<_>>()
        .join("  ");

    let mut trend_lines = vec![
        Line::from(vec![
            Span::styled("Temp   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&temp_values, chart_width),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Precip ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&precip_values, chart_width),
                Style::default().fg(theme.info),
            ),
        ]),
    ];
    if trend_area.height >= 3 {
        trend_lines.push(Line::from(vec![
            Span::styled("Next   ", Style::default().fg(theme.muted_text)),
            Span::styled(hours_preview, Style::default().fg(theme.text)),
        ]));
    }
    if trend_area.height >= 4 {
        let humidity_values = weather
            .hourly
            .iter()
            .take(24)
            .filter_map(|h| h.relative_humidity_2m)
            .collect::<Vec<_>>();
        trend_lines.push(Line::from(vec![
            Span::styled("Hum    ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&humidity_values, chart_width),
                Style::default().fg(theme.success),
            ),
        ]));
    }

    let remaining = trend_area.height.saturating_sub(trend_lines.len() as u16) as usize;
    let detail_rows = remaining.min(8);
    for h in weather.hourly.iter().skip(1).take(detail_rows) {
        let temp = h
            .temperature_2m_c
            .map(|v| format!("{:>3}°", round_temp(convert_temp(v, state.units))))
            .unwrap_or_else(|| " --°".to_string());
        let precip = h
            .precipitation_probability
            .map(|p| format!("P{:>2}%", p.round() as i32))
            .unwrap_or_else(|| "P--%".to_string());
        let wx = weather_icon(h.weather_code.unwrap_or(code), state.settings.icon_mode);
        trend_lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", h.time.format("%H:%M")),
                Style::default().fg(theme.muted_text),
            ),
            Span::styled(wx, Style::default().fg(theme.accent)),
            Span::raw("  "),
            Span::styled(temp, Style::default().fg(theme.text)),
            Span::raw("  "),
            Span::styled(precip, Style::default().fg(theme.info)),
        ]));
    }
    frame.render_widget(Paragraph::new(trend_lines), trend_area);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeroScale {
    Compact,
    Standard,
    Deluxe,
}

impl HeroScale {
    fn for_area(area: Rect) -> Self {
        if area.width >= 84 && area.height >= 14 {
            Self::Deluxe
        } else if area.width >= 56 && area.height >= 9 {
            Self::Standard
        } else {
            Self::Compact
        }
    }

    fn compact_metrics(self) -> bool {
        matches!(self, Self::Compact)
    }

    fn metric_gap(self) -> &'static str {
        if matches!(self, Self::Deluxe) {
            "    "
        } else {
            "  "
        }
    }

    fn chart_left_padding(self) -> u16 {
        if matches!(self, Self::Deluxe) { 10 } else { 14 }
    }

    fn chart_max_width(self) -> u16 {
        if matches!(self, Self::Deluxe) { 64 } else { 48 }
    }
}

fn sparkline(values: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if width == 0 {
        return String::new();
    }
    if values.is_empty() {
        return "·".repeat(width);
    }

    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);

    (0..width)
        .map(|i| {
            let idx = (i * values.len() / width).min(values.len() - 1);
            let v = values[idx];
            let normalized = ((v - min) / span).clamp(0.0, 1.0);
            let level = (normalized * (BARS.len() - 1) as f32).round() as usize;
            BARS[level]
        })
        .collect()
}

fn render_landmark(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    is_day: bool,
    theme: crate::ui::theme::Theme,
    capability: ColorCapability,
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
                            colored_lines: None,
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
    let scene_label = scene.label.clone();
    let scene_lines = scene.lines;
    let colored_lines = scene.colored_lines;
    let has_colored = colored_lines.is_some();

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} {}",
            if state.animate_ui { "~>" } else { "--" },
            scene_label
        ),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )));

    if let Some(color_rows) = colored_lines {
        for row in color_rows {
            lines.push(colored_line_from_glyphs(
                &row,
                tint,
                theme.accent,
                capability,
            ));
        }
    } else {
        for line in scene_lines {
            lines.push(Line::from(Span::styled(line, Style::default().fg(tint))));
        }
    }

    let mut text = Text::from(lines);
    if !has_colored {
        text = text.patch_style(Style::default().fg(tint));
    }
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn colored_line_from_glyphs(
    glyphs: &[ColoredGlyph],
    fallback: Color,
    theme_tint: Color,
    capability: ColorCapability,
) -> Line<'static> {
    if glyphs.is_empty() {
        return Line::from("");
    }

    let fallback_rgb = color_to_rgb(fallback);
    let theme_rgb = color_to_rgb(theme_tint);
    let themed = |glyph: &ColoredGlyph| -> Color {
        let base = glyph.color.unwrap_or(fallback_rgb);
        let with_theme = blend_rgb(
            base,
            theme_rgb,
            if glyph.color.is_some() { 0.34 } else { 0.58 },
        );
        let stabilized = blend_rgb(
            with_theme,
            fallback_rgb,
            if glyph.color.is_some() { 0.14 } else { 0.22 },
        );
        quantize(
            Color::Rgb(stabilized.0, stabilized.1, stabilized.2),
            capability,
        )
    };

    let mut spans = Vec::new();
    let mut run = String::new();
    let mut current = themed(&glyphs[0]);

    for glyph in glyphs {
        let next = themed(glyph);
        if next != current {
            spans.push(Span::styled(
                std::mem::take(&mut run),
                Style::default().fg(current),
            ));
            current = next;
        }
        run.push(glyph.ch);
    }

    if !run.is_empty() {
        spans.push(Span::styled(run, Style::default().fg(current)));
    }

    Line::from(spans)
}

fn render_loading_choreography(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: crate::ui::theme::Theme,
    scale: HeroScale,
) {
    let stage_idx = loading_stage_index(state.frame_tick);
    let spinner = loading_spinner(state.frame_tick);
    let bar = indeterminate_bar(
        state.frame_tick,
        match scale {
            HeroScale::Compact => 18,
            HeroScale::Standard => 24,
            HeroScale::Deluxe => 34,
        },
    );

    let stage_labels = [
        "Locate city context",
        "Fetch weather layers",
        "Compose ambient scene",
    ];
    let mut stage_spans = Vec::new();
    for (idx, label) in stage_labels.into_iter().enumerate() {
        let (marker, color) = if idx < stage_idx {
            ("● ", theme.success)
        } else if idx == stage_idx {
            ("◉ ", theme.accent)
        } else {
            ("○ ", theme.muted_text)
        };
        stage_spans.push(Span::styled(marker, Style::default().fg(color)));
        stage_spans.push(Span::styled(label, Style::default().fg(color)));
        if idx + 1 < stage_labels.len() {
            stage_spans.push(Span::raw("   "));
        }
    }

    let mut lines = vec![
        Line::from(Span::styled(
            format!("{spinner} Preparing atmosphere"),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(stage_spans),
        Line::from(Span::styled(bar, Style::default().fg(theme.info))),
        Line::from(Span::styled(
            state.loading_message.clone(),
            Style::default().fg(theme.text),
        )),
    ];

    if area.height >= 9 {
        lines.push(Line::from(""));
        let skeleton_width = usize::from(area.width).saturating_sub(4).clamp(16, 56);
        lines.push(Line::from(vec![
            Span::styled("Hero   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                loading_skeleton_row(state.frame_tick, skeleton_width, 0),
                Style::default().fg(theme.accent),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Hourly ", Style::default().fg(theme.muted_text)),
            Span::styled(
                loading_skeleton_row(state.frame_tick, skeleton_width, 1),
                Style::default().fg(theme.info),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Daily  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                loading_skeleton_row(state.frame_tick, skeleton_width, 2),
                Style::default().fg(theme.success),
            ),
        ]));
    }

    lines.push(Line::from(Span::styled(
        "Tip: press l for cities, s for settings, r to retry, q to quit",
        Style::default().fg(theme.muted_text),
    )));

    frame.render_widget(Paragraph::new(lines), area);
}

fn loading_skeleton_row(frame_tick: u64, width: usize, lane: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut chars = vec!['·'; width];
    let head = ((frame_tick as usize) + lane * 5) % width;
    chars[head] = '█';
    if head > 0 {
        chars[head - 1] = '▓';
    }
    if head + 1 < width {
        chars[head + 1] = '▓';
    }
    if head + 2 < width {
        chars[head + 2] = '▒';
    }
    chars.into_iter().collect()
}

fn loading_spinner(frame_tick: u64) -> &'static str {
    const FRAMES: [&str; 8] = ["-", "\\", "|", "/", "-", "\\", "|", "/"];
    FRAMES[(frame_tick as usize) % FRAMES.len()]
}

fn loading_stage_index(frame_tick: u64) -> usize {
    ((frame_tick / 14) as usize) % 3
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
    flash_bg: Color,
    flash_fg: Color,
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
                    cell.set_bg(self.flash_bg).set_fg(self.flash_fg);
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

fn blend_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::Gray => (229, 229, 229),
        Color::DarkGray => (102, 102, 102),
        Color::LightRed => (241, 76, 76),
        Color::LightGreen => (35, 209, 139),
        Color::LightYellow => (245, 245, 67),
        Color::LightBlue => (59, 142, 234),
        Color::LightMagenta => (214, 112, 214),
        Color::LightCyan => (41, 184, 219),
        Color::White => (255, 255, 255),
        Color::Indexed(idx) => indexed_color_to_rgb(idx),
        Color::Reset => (255, 255, 255),
    }
}

fn indexed_color_to_rgb(idx: u8) -> (u8, u8, u8) {
    const ANSI: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 49, 49),
        (13, 188, 121),
        (229, 229, 16),
        (36, 114, 200),
        (188, 63, 188),
        (17, 168, 205),
        (229, 229, 229),
        (102, 102, 102),
        (241, 76, 76),
        (35, 209, 139),
        (245, 245, 67),
        (59, 142, 234),
        (214, 112, 214),
        (41, 184, 219),
        (255, 255, 255),
    ];

    if idx < 16 {
        return ANSI[idx as usize];
    }
    if idx <= 231 {
        let n = idx - 16;
        let r = n / 36;
        let g = (n % 36) / 6;
        let b = n % 6;
        let convert = |v: u8| -> u8 { if v == 0 { 0 } else { 55 + v * 40 } };
        return (convert(r), convert(g), convert(b));
    }
    let gray = 8 + (idx - 232) * 10;
    (gray, gray, gray)
}

fn inset_rect(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    let h = horizontal.min(area.width.saturating_sub(1) / 2);
    let v = vertical.min(area.height.saturating_sub(1) / 2);
    Rect {
        x: area.x.saturating_add(h),
        y: area.y.saturating_add(v),
        width: area.width.saturating_sub(h.saturating_mul(2)),
        height: area.height.saturating_sub(v.saturating_mul(2)),
    }
}

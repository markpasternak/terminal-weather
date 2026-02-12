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
    cli::{Cli, HeroVisualArg},
    domain::weather::{
        HourlyForecast, WeatherCategory, convert_temp, round_temp, weather_code_to_category,
        weather_label,
    },
    ui::theme::{condition_color, detect_color_capability, theme_for},
    ui::widgets::landmark::{
        LandmarkTint, scene_for_gauge_cluster, scene_for_sky_observatory, scene_for_weather,
    },
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
        render_landmark(frame, right_content, state, is_day, theme);
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
        let dew = round_temp(convert_temp(weather.current.dew_point_2m_c, state.units));
        let wind_dir = compass(weather.current.wind_direction_10m);
        let wind = weather.current.wind_speed_10m.round() as i32;
        let gust = weather.current.wind_gusts_10m.round() as i32;
        let visibility = format_visibility(weather.current.visibility_m);
        let pressure = weather.current.pressure_msl_hpa.round() as i32;
        let pressure_trend = pressure_trend_marker(&weather.hourly);
        let uv_today = weather
            .daily
            .first()
            .and_then(|d| d.uv_index_max)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "--".to_string());
        let cloud_total = weather.current.cloud_cover.round() as i32;
        let (cloud_low, cloud_mid, cloud_high) =
            cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
        let cloud_split = format_cloud_layers(cloud_low, cloud_mid, cloud_high);

        if compact_metrics {
            lines.push(Line::from(vec![
                Span::styled("Wind ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{wind}/{gust} km/h {wind_dir}"),
                    Style::default().fg(theme.success),
                ),
                Span::raw(metric_gap),
                Span::styled("P ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{pressure}{pressure_trend}"),
                    Style::default().fg(theme.warning),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Dew ", Style::default().fg(muted_color)),
                Span::styled(format!("{dew}°"), Style::default().fg(theme.text)),
                Span::raw(metric_gap),
                Span::styled("Vis ", Style::default().fg(muted_color)),
                Span::styled(visibility, Style::default().fg(theme.info)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Feels ", Style::default().fg(muted_color)),
                Span::styled(format!("{feels}°"), Style::default().fg(text_color)),
                Span::raw(metric_gap),
                Span::styled("Dew ", Style::default().fg(muted_color)),
                Span::styled(format!("{dew}°"), Style::default().fg(theme.info)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Wind ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{wind}/{gust} km/h {wind_dir}"),
                    Style::default().fg(theme.success),
                ),
                Span::raw(metric_gap),
                Span::styled("Vis ", Style::default().fg(muted_color)),
                Span::styled(visibility, Style::default().fg(theme.accent)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("P ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{pressure}hPa{pressure_trend}"),
                    Style::default().fg(theme.warning),
                ),
                Span::raw(metric_gap),
                Span::styled("Hum ", Style::default().fg(muted_color)),
                Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Cloud ", Style::default().fg(muted_color)),
                Span::styled(
                    format!("{cloud_total}%"),
                    Style::default().fg(theme.landmark_neutral),
                ),
                Span::raw(" "),
                Span::styled(cloud_split, Style::default().fg(theme.muted_text)),
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
    let dew = round_temp(convert_temp(weather.current.dew_point_2m_c, state.units));
    let wind_dir = compass(weather.current.wind_direction_10m);
    let wind = weather.current.wind_speed_10m.round() as i32;
    let gust = weather.current.wind_gusts_10m.round() as i32;
    let visibility = format_visibility(weather.current.visibility_m);
    let pressure = weather.current.pressure_msl_hpa.round() as i32;
    let pressure_trend = pressure_trend_marker(&weather.hourly);
    let uv_today = weather
        .daily
        .first()
        .and_then(|d| d.uv_index_max)
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "--".to_string());
    let cloud_total = weather.current.cloud_cover.round() as i32;
    let (cloud_low, cloud_mid, cloud_high) =
        cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
    let cloud_split = format_cloud_layers(cloud_low, cloud_mid, cloud_high);

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
            Span::styled("Dew ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{dew}°"), Style::default().fg(theme.info)),
        ]),
        Line::from(vec![
            Span::styled("Wind ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{wind}/{gust} km/h {wind_dir}"),
                Style::default().fg(theme.success),
            ),
            Span::raw("  "),
            Span::styled("Vis ", Style::default().fg(theme.muted_text)),
            Span::styled(visibility, Style::default().fg(theme.accent)),
        ]),
    ];
    frame.render_widget(Paragraph::new(left_metrics), metric_cols[0]);

    let right_metrics = vec![
        Line::from(vec![
            Span::styled("P ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{pressure}hPa{pressure_trend}"),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Hum ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{humidity}%"), Style::default().fg(theme.info)),
        ]),
        Line::from(vec![
            Span::styled("Cloud ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{cloud_total}%"),
                Style::default().fg(theme.landmark_neutral),
            ),
            Span::raw(" "),
            Span::styled(cloud_split, Style::default().fg(theme.muted_text)),
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
    let pressure_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.pressure_msl_hpa)
        .collect::<Vec<_>>();
    let gust_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.wind_gusts_10m)
        .collect::<Vec<_>>();
    let precip_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.precipitation_mm)
        .collect::<Vec<_>>();
    let cloud_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.cloud_cover)
        .collect::<Vec<_>>();
    let visibility_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.visibility_m.map(|m| m / 1000.0))
        .collect::<Vec<_>>();

    let mut trend_lines = vec![
        Line::from(vec![
            Span::styled("Temp   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&temp_values, chart_width),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Press  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&pressure_values, chart_width),
                Style::default().fg(theme.info),
            ),
        ]),
    ];
    if trend_area.height >= 3 {
        trend_lines.push(Line::from(vec![
            Span::styled("Gust   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&gust_values, chart_width),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
    if trend_area.height >= 4 {
        trend_lines.push(Line::from(vec![
            Span::styled("Precip ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&precip_values, chart_width),
                Style::default().fg(theme.info),
            ),
        ]));
    }
    if trend_area.height >= 5 {
        trend_lines.push(Line::from(vec![
            Span::styled("Cloud  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&cloud_values, chart_width),
                Style::default().fg(theme.landmark_neutral),
            ),
        ]));
    }
    if trend_area.height >= 6 {
        trend_lines.push(Line::from(vec![
            Span::styled("Vis km ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&visibility_values, chart_width),
                Style::default().fg(theme.success),
            ),
        ]));
    }
    if trend_area.height >= 7
        && let Some((min_temp, max_temp)) = value_span(&temp_values)
    {
        trend_lines.push(Line::from(vec![
            Span::styled("24h span ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{}°..{}°", round_temp(min_temp), round_temp(max_temp)),
                Style::default().fg(theme.accent),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Δ{}°", round_temp(max_temp - min_temp)),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
    if trend_area.height >= 8 {
        trend_lines.push(Line::from(vec![
            Span::styled("Next rain ", Style::default().fg(theme.muted_text)),
            Span::styled(
                next_precip_summary(&weather.hourly),
                Style::default().fg(theme.info),
            ),
        ]));
    }
    if trend_area.height >= 9 {
        trend_lines.push(Line::from(vec![
            Span::styled("Peak gust ", Style::default().fg(theme.muted_text)),
            Span::styled(
                peak_gust_summary(&weather.hourly),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
    if trend_area.height >= 10 {
        trend_lines.push(Line::from(vec![
            Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
            Span::styled(
                pressure_span_summary(&pressure_values),
                Style::default().fg(theme.success),
            ),
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
        if matches!(self, Self::Deluxe) { 10 } else { 12 }
    }

    fn chart_max_width(self) -> u16 {
        if matches!(self, Self::Deluxe) {
            120
        } else {
            72
        }
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
) {
    if area.width < 10 || area.height < 4 {
        return;
    }

    let scene = match state.settings.hero_visual {
        HeroVisualArg::AtmosCanvas => {
            if let Some(bundle) = state.weather.as_ref() {
                scene_for_weather(
                    bundle,
                    state.frame_tick,
                    state.animate_ui,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            } else {
                loading_scene("Atmos Canvas", area.width, area.height, is_day)
            }
        }
        HeroVisualArg::GaugeCluster => {
            if let Some(bundle) = state.weather.as_ref() {
                scene_for_gauge_cluster(
                    bundle,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            } else {
                loading_scene("Gauge Cluster", area.width, area.height, is_day)
            }
        }
        HeroVisualArg::SkyObservatory => {
            if let Some(bundle) = state.weather.as_ref() {
                scene_for_sky_observatory(
                    bundle,
                    state.frame_tick,
                    state.animate_ui,
                    area.width.saturating_sub(2),
                    area.height.saturating_sub(2),
                )
            } else {
                loading_scene("Sky Observatory", area.width, area.height, is_day)
            }
        }
    };

    let tint = match scene.tint {
        LandmarkTint::Warm => theme.landmark_warm,
        LandmarkTint::Cool => theme.landmark_cool,
        LandmarkTint::Neutral => theme.landmark_neutral,
    };
    let scene_label = scene.label;
    let scene_lines = scene.lines;

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} {}",
            if state.animate_ui { "~>" } else { "--" },
            scene_label
        ),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )));

    for line in scene_lines {
        lines.push(Line::from(Span::styled(line, Style::default().fg(tint))));
    }

    let text = Text::from(lines).patch_style(Style::default().fg(tint));
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
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

fn format_visibility(meters: f32) -> String {
    if !meters.is_finite() || meters <= 0.0 {
        return "--".to_string();
    }
    let km = meters / 1000.0;
    if km >= 20.0 {
        format!("{km:.0}km")
    } else {
        format!("{km:.1}km")
    }
}

fn pressure_trend_marker(hourly: &[HourlyForecast]) -> &'static str {
    let mut values = hourly.iter().take(6).filter_map(|h| h.pressure_msl_hpa);
    let Some(start) = values.next() else {
        return "";
    };
    let end = values.next_back().unwrap_or(start);
    let delta = end - start;
    if delta >= 1.2 {
        "↗"
    } else if delta <= -1.2 {
        "↘"
    } else {
        "→"
    }
}

fn cloud_layers_from_hourly(
    hourly: &[HourlyForecast],
) -> Option<(Option<f32>, Option<f32>, Option<f32>)> {
    let mut low = Vec::new();
    let mut mid = Vec::new();
    let mut high = Vec::new();
    for hour in hourly.iter().take(8) {
        if let Some(v) = hour.cloud_cover_low {
            low.push(v);
        }
        if let Some(v) = hour.cloud_cover_mid {
            mid.push(v);
        }
        if let Some(v) = hour.cloud_cover_high {
            high.push(v);
        }
    }

    let low_avg = average(&low);
    let mid_avg = average(&mid);
    let high_avg = average(&high);
    if low_avg.is_none() && mid_avg.is_none() && high_avg.is_none() {
        None
    } else {
        Some((low_avg, mid_avg, high_avg))
    }
}

fn format_cloud_layers(low: Option<f32>, mid: Option<f32>, high: Option<f32>) -> String {
    format!(
        "{}/{}/{}%",
        format_pct(low),
        format_pct(mid),
        format_pct(high)
    )
}

fn format_pct(value: Option<f32>) -> String {
    value
        .map(|v| format!("{:>2}", v.round() as i32))
        .unwrap_or_else(|| "--".to_string())
}

fn average(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>() / values.len() as f32)
    }
}

fn value_span(values: &[f32]) -> Option<(f32, f32)> {
    if values.is_empty() {
        return None;
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    Some((min, max))
}

fn next_precip_summary(hourly: &[HourlyForecast]) -> String {
    let threshold = 0.2_f32;
    for (idx, h) in hourly.iter().take(12).enumerate() {
        let amount = h.precipitation_mm.unwrap_or(0.0).max(0.0);
        if amount >= threshold {
            if idx == 0 {
                return format!("now ({amount:.1}mm)");
            }
            return format!("in {idx}h ({amount:.1}mm)");
        }
    }
    "none in 12h".to_string()
}

fn peak_gust_summary(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(24)
        .filter_map(|h| h.wind_gusts_10m.map(|g| (g, h.time)))
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(gust, time)| format!("{}km/h @ {}", gust.round() as i32, time.format("%H:%M")))
        .unwrap_or_else(|| "--".to_string())
}

fn pressure_span_summary(values: &[f32]) -> String {
    value_span(values)
        .map(|(min, max)| format!("{:.0}..{:.0}hPa", min, max))
        .unwrap_or_else(|| "--".to_string())
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

fn loading_scene(
    name: &str,
    width: u16,
    height: u16,
    is_day: bool,
) -> crate::ui::widgets::landmark::LandmarkScene {
    let label = format!("{name} · waiting for weather");
    let icon = if is_day { "o" } else { "*" };
    let mut lines = vec![
        " ".repeat(width as usize),
        format!("   preparing {name}"),
        format!("   collecting forecast lanes {icon}"),
    ];
    while lines.len() < height as usize {
        lines.push(" ".repeat(width as usize));
    }
    crate::ui::widgets::landmark::LandmarkScene {
        label,
        lines: lines
            .into_iter()
            .map(|line| {
                let mut s = line.chars().take(width as usize).collect::<String>();
                let len = s.chars().count();
                if len < width as usize {
                    s.push_str(&" ".repeat(width as usize - len));
                }
                s
            })
            .collect(),
        tint: LandmarkTint::Neutral,
    }
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

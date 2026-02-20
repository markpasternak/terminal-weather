use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::state::AppState,
    domain::weather::{
        ForecastBundle, HourlyForecast, convert_temp, round_temp, weather_code_to_category,
        weather_label_for_time,
    },
    ui::theme::{Theme, condition_color},
};

use super::weather::{
    HeroScale, cloud_layers_from_hourly, compass, format_cloud_layers, format_visibility,
    pressure_trend_marker,
};

pub fn render_weather_info_expanded(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
    weather: &ForecastBundle,
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
    let condition = weather_label_for_time(code, weather.current.is_day);
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
            Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
            Span::styled(visibility, Style::default().fg(theme.accent)),
        ]),
    ];
    frame.render_widget(Paragraph::new(left_metrics), metric_cols[0]);

    let right_metrics = vec![
        Line::from(vec![
            Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{pressure}hPa{pressure_trend}"),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Humidity ", Style::default().fg(theme.muted_text)),
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

pub fn value_span(values: &[f32]) -> Option<(f32, f32)> {
    if values.is_empty() {
        return None;
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    Some((min, max))
}

pub fn next_precip_summary(hourly: &[HourlyForecast]) -> String {
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

pub fn peak_gust_summary(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(24)
        .filter_map(|h| h.wind_gusts_10m.map(|g| (g, h.time)))
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(gust, time)| format!("{}km/h @ {}", gust.round() as i32, time.format("%H:%M")))
        .unwrap_or_else(|| "--".to_string())
}

pub fn pressure_span_summary(values: &[f32]) -> String {
    value_span(values)
        .map(|(min, max)| format!("{:.0}..{:.0}hPa", min, max))
        .unwrap_or_else(|| "--".to_string())
}

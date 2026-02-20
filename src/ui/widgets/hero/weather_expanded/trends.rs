use super::*;

pub(super) fn collect_trend_series(
    weather: &ForecastBundle,
    units: crate::domain::weather::Units,
    trend_area: Rect,
    scale: HeroScale,
) -> ExpandedTrendsData {
    let chart_width = trend_area
        .width
        .saturating_sub(scale.chart_left_padding())
        .clamp(12, scale.chart_max_width()) as usize;
    let temp_values = collect_hourly_series(&weather.hourly, |hour| temp_series_value(hour, units));
    let pressure_values = collect_hourly_series(&weather.hourly, pressure_series_value);
    let gust_values = collect_hourly_series(&weather.hourly, gust_series_value);
    let precip_values = collect_hourly_series(&weather.hourly, precip_series_value);
    let cloud_values = collect_hourly_series(&weather.hourly, cloud_series_value);
    let visibility_values = collect_hourly_series(&weather.hourly, visibility_series_value);

    ExpandedTrendsData {
        chart_width,
        temp_values,
        pressure_values,
        gust_values,
        precip_values,
        cloud_values,
        visibility_values,
    }
}

fn collect_hourly_series(
    hourly: &[HourlyForecast],
    projection: impl Fn(&HourlyForecast) -> Option<f32>,
) -> Vec<f32> {
    hourly
        .iter()
        .take(24)
        .filter_map(projection)
        .collect::<Vec<_>>()
}

fn temp_series_value(hour: &HourlyForecast, units: crate::domain::weather::Units) -> Option<f32> {
    hour.temperature_2m_c
        .map(|value| convert_temp(value, units))
}

fn pressure_series_value(hour: &HourlyForecast) -> Option<f32> {
    hour.pressure_msl_hpa
}

fn gust_series_value(hour: &HourlyForecast) -> Option<f32> {
    hour.wind_gusts_10m
}

fn precip_series_value(hour: &HourlyForecast) -> Option<f32> {
    hour.precipitation_mm
}

fn cloud_series_value(hour: &HourlyForecast) -> Option<f32> {
    hour.cloud_cover
}

fn visibility_series_value(hour: &HourlyForecast) -> Option<f32> {
    hour.visibility_m.map(|value| value / 1000.0)
}

pub(super) fn build_expanded_trend_lines(
    data: &ExpandedTrendsData,
    trend_height: u16,
    weather: &ForecastBundle,
    theme: Theme,
) -> Vec<Line<'static>> {
    let mut trend_lines = vec![
        trend_line(
            "Temp   ",
            &data.temp_values,
            data.chart_width,
            theme.accent,
            theme,
        ),
        trend_line(
            "Press  ",
            &data.pressure_values,
            data.chart_width,
            theme.info,
            theme,
        ),
    ];

    append_optional_trend_lines(&mut trend_lines, data, trend_height, theme);
    append_trend_summary_lines(&mut trend_lines, data, trend_height, weather, theme);
    trend_lines
}

fn trend_line(
    label: &'static str,
    values: &[f32],
    width: usize,
    color: Color,
    theme: Theme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(theme.muted_text)),
        Span::styled(sparkline(values, width), Style::default().fg(color)),
    ])
}

fn append_optional_trend_lines(
    lines: &mut Vec<Line<'static>>,
    data: &ExpandedTrendsData,
    trend_height: u16,
    theme: Theme,
) {
    let options: [(u16, &str, &[f32], Color); 4] = [
        (3, "Gust   ", &data.gust_values, theme.warning),
        (4, "Precip ", &data.precip_values, theme.info),
        (5, "Cloud  ", &data.cloud_values, theme.landmark_neutral),
        (6, "Vis km ", &data.visibility_values, theme.success),
    ];
    for (min_height, label, values, color) in options {
        if trend_height >= min_height {
            lines.push(trend_line(label, values, data.chart_width, color, theme));
        }
    }
}

fn append_trend_summary_lines(
    lines: &mut Vec<Line<'static>>,
    data: &ExpandedTrendsData,
    trend_height: u16,
    weather: &ForecastBundle,
    theme: Theme,
) {
    if trend_height >= 7 {
        append_temp_span_line(lines, &data.temp_values, theme);
    }
    if trend_height >= 8 {
        append_next_precip_line(lines, weather, theme);
    }
    if trend_height >= 9 {
        append_peak_gust_line(lines, weather, theme);
    }
    if trend_height >= 10 {
        append_pressure_span_line(lines, &data.pressure_values, theme);
    }
}

fn append_temp_span_line(lines: &mut Vec<Line<'static>>, values: &[f32], theme: Theme) {
    if let Some((min_temp, max_temp)) = value_span(values) {
        lines.push(Line::from(vec![
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
}

fn append_next_precip_line(lines: &mut Vec<Line<'static>>, weather: &ForecastBundle, theme: Theme) {
    lines.push(Line::from(vec![
        Span::styled("Next precip ", Style::default().fg(theme.muted_text)),
        Span::styled(
            next_precip_summary(&weather.hourly),
            Style::default().fg(theme.info),
        ),
    ]));
}

fn append_peak_gust_line(lines: &mut Vec<Line<'static>>, weather: &ForecastBundle, theme: Theme) {
    lines.push(Line::from(vec![
        Span::styled("Peak gust ", Style::default().fg(theme.muted_text)),
        Span::styled(
            peak_gust_summary(&weather.hourly),
            Style::default().fg(theme.warning),
        ),
    ]));
}

fn append_pressure_span_line(lines: &mut Vec<Line<'static>>, values: &[f32], theme: Theme) {
    lines.push(Line::from(vec![
        Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
        Span::styled(
            pressure_span_summary(values),
            Style::default().fg(theme.success),
        ),
    ]));
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

pub(super) fn value_span(values: &[f32]) -> Option<(f32, f32)> {
    if values.is_empty() {
        return None;
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    Some((min, max))
}

pub(super) fn next_precip_summary(hourly: &[HourlyForecast]) -> String {
    crate::domain::weather::summarize_precip_window(
        hourly,
        crate::domain::weather::PRECIP_NEAR_TERM_HOURS,
        crate::domain::weather::PRECIP_SIGNIFICANT_THRESHOLD_MM,
    )
    .map_or_else(
        || {
            format!(
                "none in {}h",
                crate::domain::weather::PRECIP_NEAR_TERM_HOURS
            )
        },
        |summary| {
            if summary.has_precip_now() {
                format!("now ({:.1}mm)", summary.first_amount_mm)
            } else {
                format!(
                    "in {}h ({:.1}mm)",
                    summary.first_idx, summary.first_amount_mm
                )
            }
        },
    )
}

pub(super) fn peak_gust_summary(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(24)
        .filter_map(|h| h.wind_gusts_10m.map(|g| (g, h.time)))
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(gust, time)| format!("{}m/s @ {}", round_wind_speed(gust), time.format("%H:%M")))
        .unwrap_or_else(|| "--".to_string())
}

pub(super) fn pressure_span_summary(values: &[f32]) -> String {
    value_span(values).map_or_else(
        || "--".to_string(),
        |(min, max)| format!("{min:.0}..{max:.0}hPa"),
    )
}

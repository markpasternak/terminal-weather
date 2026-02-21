use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{domain::weather::AirQualityCategory, ui::theme::Theme};

#[derive(Debug)]
pub(super) struct WeatherMetricsData {
    pub(super) feels: i32,
    pub(super) humidity: i32,
    pub(super) dew: i32,
    pub(super) wind_dir: &'static str,
    pub(super) wind: i32,
    pub(super) gust: i32,
    pub(super) visibility: String,
    pub(super) pressure: i32,
    pub(super) pressure_trend: &'static str,
    pub(super) uv_today: String,
    pub(super) cloud_total: i32,
    pub(super) cloud_split: String,
    pub(super) precip_probability: String,
    pub(super) aqi: String,
    pub(super) aqi_category: AirQualityCategory,
    pub(super) aqi_available: bool,
}

pub(super) fn push_metric_lines(
    lines: &mut Vec<Line<'static>>,
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
    compact: bool,
) {
    if compact {
        push_compact_metric_lines(lines, data, theme, metric_gap);
    } else {
        push_standard_metric_lines(lines, data, theme, metric_gap);
    }
}

fn push_compact_metric_lines(
    lines: &mut Vec<Line<'static>>,
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) {
    lines.push(Line::from(vec![
        Span::styled("Wind ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}/{} m/s {}", data.wind, data.gust, data.wind_dir),
            Style::default().fg(theme.success),
        ),
        Span::raw(metric_gap),
        Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
        Span::styled(data.visibility.clone(), Style::default().fg(theme.accent)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}{}", data.pressure, data.pressure_trend),
            Style::default().fg(theme.warning),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Dew ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("{}°", data.dew), Style::default().fg(theme.text)),
        Span::raw(metric_gap),
        Span::styled("Humidity ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}%", data.humidity),
            Style::default().fg(theme.info),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Rain chance ", Style::default().fg(theme.muted_text)),
        Span::styled(
            data.precip_probability.clone(),
            Style::default().fg(theme.info),
        ),
        Span::raw(metric_gap),
        Span::styled("AQI ", Style::default().fg(theme.muted_text)),
        Span::styled(
            data.aqi.clone(),
            Style::default().fg(aqi_color(data, theme)),
        ),
    ]));
}

fn push_standard_metric_lines(
    lines: &mut Vec<Line<'static>>,
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) {
    lines.push(standard_metric_feels_line(data, theme, metric_gap));
    lines.push(standard_metric_wind_line(data, theme, metric_gap));
    lines.push(standard_metric_pressure_line(data, theme, metric_gap));
    lines.push(standard_metric_cloud_line(data, theme, metric_gap));
    lines.push(standard_metric_risk_line(data, theme, metric_gap));
}

fn standard_metric_feels_line(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("Feels ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("{}°", data.feels), Style::default().fg(theme.text)),
        Span::raw(metric_gap),
        Span::styled("Dew ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("{}°", data.dew), Style::default().fg(theme.info)),
    ])
}

fn standard_metric_wind_line(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("Wind ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}/{} m/s {}", data.wind, data.gust, data.wind_dir),
            Style::default().fg(theme.success),
        ),
        Span::raw(metric_gap),
        Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
        Span::styled(data.visibility.clone(), Style::default().fg(theme.accent)),
    ])
}

fn standard_metric_pressure_line(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}hPa{}", data.pressure, data.pressure_trend),
            Style::default().fg(theme.warning),
        ),
        Span::raw(metric_gap),
        Span::styled("Humidity ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}%", data.humidity),
            Style::default().fg(theme.info),
        ),
    ])
}

fn standard_metric_cloud_line(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("Cloud ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}%", data.cloud_total),
            Style::default().fg(theme.landmark_neutral),
        ),
        Span::raw(" "),
        Span::styled(
            data.cloud_split.clone(),
            Style::default().fg(theme.muted_text),
        ),
        Span::raw(metric_gap),
        Span::styled("UV ", Style::default().fg(theme.muted_text)),
        Span::styled(data.uv_today.clone(), Style::default().fg(theme.warning)),
    ])
}

fn standard_metric_risk_line(
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("Rain chance ", Style::default().fg(theme.muted_text)),
        Span::styled(
            data.precip_probability.clone(),
            Style::default().fg(theme.info),
        ),
        Span::raw(metric_gap),
        Span::styled("AQI ", Style::default().fg(theme.muted_text)),
        Span::styled(
            data.aqi.clone(),
            Style::default().fg(aqi_color(data, theme)),
        ),
    ])
}

fn aqi_color(data: &WeatherMetricsData, theme: Theme) -> Color {
    super::hero_shared::aqi_color(theme, data.aqi_category, data.aqi_available)
}

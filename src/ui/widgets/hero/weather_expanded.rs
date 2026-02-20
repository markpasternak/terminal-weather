use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
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

#[derive(Debug)]
struct ExpandedTopData {
    temp: i32,
    unit_symbol: &'static str,
    condition: String,
    condition_color: Color,
    location: String,
    high_low: Option<(i32, i32)>,
    freshness: &'static str,
    freshness_color: Color,
    updated: String,
}

#[derive(Debug)]
struct ExpandedMetricsData {
    feels: i32,
    dew: i32,
    wind_dir: String,
    wind: i32,
    gust: i32,
    visibility: String,
    pressure: i32,
    pressure_trend: &'static str,
    humidity: i32,
    cloud_total: i32,
    cloud_split: String,
    uv_today: String,
    sunrise: String,
    sunset: String,
}

#[derive(Debug)]
struct ExpandedTrendsData {
    chart_width: usize,
    temp_values: Vec<f32>,
    pressure_values: Vec<f32>,
    gust_values: Vec<f32>,
    precip_values: Vec<f32>,
    cloud_values: Vec<f32>,
    visibility_values: Vec<f32>,
}

pub fn render_weather_info_expanded(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
    weather: &ForecastBundle,
    code: u8,
) {
    let scale = HeroScale::for_area(area);
    let sections = expanded_sections(area);
    let top_area = sections[0];
    let metrics_area = sections[1];
    let trend_area = sections[2];

    let top_data = build_expanded_top_data(state, weather, theme, code);
    let metrics_data = build_expanded_metrics_data(state, weather);

    let trends_data = collect_trend_series(weather, state.units, trend_area, scale);

    frame.render_widget(
        Paragraph::new(build_expanded_top_lines(&top_data, theme)),
        top_area,
    );

    let metric_cols = metric_sections(metrics_area, scale);
    let (left_metrics, right_metrics) = build_expanded_metric_lines(&metrics_data, theme);
    frame.render_widget(Paragraph::new(left_metrics), metric_cols[0]);
    frame.render_widget(Paragraph::new(right_metrics), metric_cols[1]);

    frame.render_widget(
        Paragraph::new(build_expanded_trend_lines(
            &trends_data,
            trend_area.height,
            weather,
            theme,
        )),
        trend_area,
    );
}

fn expanded_sections(area: Rect) -> std::rc::Rc<[Rect]> {
    if area.height >= 20 {
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
    }
}

fn metric_sections(metrics_area: Rect, scale: HeroScale) -> std::rc::Rc<[Rect]> {
    Layout::horizontal(if matches!(scale, HeroScale::Deluxe) {
        [Constraint::Percentage(46), Constraint::Percentage(54)]
    } else {
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    })
    .split(metrics_area)
}

fn build_expanded_top_data(
    state: &AppState,
    weather: &ForecastBundle,
    theme: Theme,
    code: u8,
) -> ExpandedTopData {
    let (freshness, freshness_color) = freshness_status(state, theme);
    ExpandedTopData {
        temp: weather.current_temp(state.units),
        unit_symbol: if state.units == crate::domain::weather::Units::Celsius {
            "C"
        } else {
            "F"
        },
        condition: weather_label_for_time(code, weather.current.is_day).to_string(),
        condition_color: condition_color(&theme, weather_code_to_category(code)),
        location: weather.location.display_name(),
        high_low: weather.high_low(state.units),
        freshness,
        freshness_color,
        updated: last_updated_label(state),
    }
}

fn freshness_status(state: &AppState, theme: Theme) -> (&'static str, Color) {
    match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => ("Fresh", theme.success),
        crate::resilience::freshness::FreshnessState::Stale => ("⚠ Stale", theme.warning),
        crate::resilience::freshness::FreshnessState::Offline => ("⚠ Offline", theme.danger),
    }
}

fn last_updated_label(state: &AppState) -> String {
    state
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
        .unwrap_or_else(|| "Last updated --:--".to_string())
}

fn build_expanded_metrics_data(state: &AppState, weather: &ForecastBundle) -> ExpandedMetricsData {
    let (cloud_low, cloud_mid, cloud_high) =
        cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
    ExpandedMetricsData {
        feels: round_temp(convert_temp(
            weather.current.apparent_temperature_c,
            state.units,
        )),
        dew: round_temp(convert_temp(weather.current.dew_point_2m_c, state.units)),
        wind_dir: compass(weather.current.wind_direction_10m).to_string(),
        wind: weather.current.wind_speed_10m.round() as i32,
        gust: weather.current.wind_gusts_10m.round() as i32,
        visibility: format_visibility(weather.current.visibility_m),
        pressure: weather.current.pressure_msl_hpa.round() as i32,
        pressure_trend: pressure_trend_marker(&weather.hourly),
        humidity: weather.current.relative_humidity_2m.round() as i32,
        cloud_total: weather.current.cloud_cover.round() as i32,
        cloud_split: format_cloud_layers(cloud_low, cloud_mid, cloud_high),
        uv_today: weather
            .daily
            .first()
            .and_then(|d| d.uv_index_max)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "--".to_string()),
        sunrise: weather
            .daily
            .first()
            .and_then(|d| d.sunrise)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string()),
        sunset: weather
            .daily
            .first()
            .and_then(|d| d.sunset)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string()),
    }
}

fn build_expanded_top_lines(data: &ExpandedTopData, theme: Theme) -> Vec<Line<'static>> {
    let mut top_lines = vec![Line::from(vec![
        Span::styled(
            format!("{}°{}  ", data.temp, data.unit_symbol),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            data.condition.clone(),
            Style::default()
                .fg(data.condition_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])];
    if let Some((high, low)) = data.high_low {
        top_lines.push(Line::from(vec![
            Span::styled(
                format!("H:{high}°  L:{low}°  "),
                Style::default().fg(theme.text),
            ),
            Span::styled(data.location.clone(), Style::default().fg(theme.muted_text)),
        ]));
    } else {
        top_lines.push(Line::from(Span::styled(
            data.location.clone(),
            Style::default().fg(theme.muted_text),
        )));
    }
    top_lines.push(Line::from(vec![
        Span::styled("Status ", Style::default().fg(theme.muted_text)),
        Span::styled(
            data.freshness,
            Style::default()
                .fg(data.freshness_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    top_lines.push(Line::from(Span::styled(
        data.updated.clone(),
        Style::default().fg(theme.muted_text),
    )));
    top_lines
}

fn build_expanded_metric_lines(
    data: &ExpandedMetricsData,
    theme: Theme,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let left_metrics = vec![
        Line::from(vec![
            Span::styled("Feels ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{}°", data.feels), Style::default().fg(theme.text)),
            Span::raw("  "),
            Span::styled("Dew ", Style::default().fg(theme.muted_text)),
            Span::styled(format!("{}°", data.dew), Style::default().fg(theme.info)),
        ]),
        Line::from(vec![
            Span::styled("Wind ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{}/{} km/h {}", data.wind, data.gust, data.wind_dir),
                Style::default().fg(theme.success),
            ),
            Span::raw("  "),
            Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
            Span::styled(data.visibility.clone(), Style::default().fg(theme.accent)),
        ]),
    ];

    let right_metrics = vec![
        Line::from(vec![
            Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{}hPa{}", data.pressure, data.pressure_trend),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Humidity ", Style::default().fg(theme.muted_text)),
            Span::styled(
                format!("{}%", data.humidity),
                Style::default().fg(theme.info),
            ),
        ]),
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
            Span::raw("  "),
            Span::styled("UV ", Style::default().fg(theme.muted_text)),
            Span::styled(data.uv_today.clone(), Style::default().fg(theme.warning)),
        ]),
        Line::from(vec![
            Span::styled("Sunrise ", Style::default().fg(theme.muted_text)),
            Span::styled(data.sunrise.clone(), Style::default().fg(theme.warning)),
            Span::raw("  "),
            Span::styled("Sunset ", Style::default().fg(theme.muted_text)),
            Span::styled(data.sunset.clone(), Style::default().fg(theme.warning)),
        ]),
    ];

    (left_metrics, right_metrics)
}

fn collect_trend_series(
    weather: &ForecastBundle,
    units: crate::domain::weather::Units,
    trend_area: Rect,
    scale: HeroScale,
) -> ExpandedTrendsData {
    let chart_width = trend_area
        .width
        .saturating_sub(scale.chart_left_padding())
        .clamp(12, scale.chart_max_width()) as usize;
    let temp_values = weather
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.temperature_2m_c.map(|v| convert_temp(v, units)))
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

fn build_expanded_trend_lines(
    data: &ExpandedTrendsData,
    trend_height: u16,
    weather: &ForecastBundle,
    theme: Theme,
) -> Vec<Line<'static>> {
    let mut trend_lines = vec![
        Line::from(vec![
            Span::styled("Temp   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.temp_values, data.chart_width),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Press  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.pressure_values, data.chart_width),
                Style::default().fg(theme.info),
            ),
        ]),
    ];
    if trend_height >= 3 {
        trend_lines.push(Line::from(vec![
            Span::styled("Gust   ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.gust_values, data.chart_width),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
    if trend_height >= 4 {
        trend_lines.push(Line::from(vec![
            Span::styled("Precip ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.precip_values, data.chart_width),
                Style::default().fg(theme.info),
            ),
        ]));
    }
    if trend_height >= 5 {
        trend_lines.push(Line::from(vec![
            Span::styled("Cloud  ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.cloud_values, data.chart_width),
                Style::default().fg(theme.landmark_neutral),
            ),
        ]));
    }
    if trend_height >= 6 {
        trend_lines.push(Line::from(vec![
            Span::styled("Vis km ", Style::default().fg(theme.muted_text)),
            Span::styled(
                sparkline(&data.visibility_values, data.chart_width),
                Style::default().fg(theme.success),
            ),
        ]));
    }
    if trend_height >= 7
        && let Some((min_temp, max_temp)) = value_span(&data.temp_values)
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
    if trend_height >= 8 {
        trend_lines.push(Line::from(vec![
            Span::styled("Next rain ", Style::default().fg(theme.muted_text)),
            Span::styled(
                next_precip_summary(&weather.hourly),
                Style::default().fg(theme.info),
            ),
        ]));
    }
    if trend_height >= 9 {
        trend_lines.push(Line::from(vec![
            Span::styled("Peak gust ", Style::default().fg(theme.muted_text)),
            Span::styled(
                peak_gust_summary(&weather.hourly),
                Style::default().fg(theme.warning),
            ),
        ]));
    }
    if trend_height >= 10 {
        trend_lines.push(Line::from(vec![
            Span::styled("Pressure ", Style::default().fg(theme.muted_text)),
            Span::styled(
                pressure_span_summary(&data.pressure_values),
                Style::default().fg(theme.success),
            ),
        ]));
    }
    trend_lines
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

#[cfg(test)]
mod tests {
    use super::{next_precip_summary, pressure_span_summary};
    use crate::domain::weather::HourlyForecast;
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn next_precip_summary_covers_now_in_nh_and_none() {
        let now = vec![hour(0, Some(0.4)), hour(1, None), hour(2, None)];
        assert_eq!(next_precip_summary(&now), "now (0.4mm)");

        let later = vec![hour(0, Some(0.0)), hour(1, Some(0.1)), hour(2, Some(0.3))];
        assert_eq!(next_precip_summary(&later), "in 2h (0.3mm)");

        let dry = vec![hour(0, Some(0.0)), hour(1, None), hour(2, Some(0.1))];
        assert_eq!(next_precip_summary(&dry), "none in 12h");
    }

    #[test]
    fn pressure_span_summary_handles_empty_and_non_empty() {
        assert_eq!(pressure_span_summary(&[]), "--");
        assert_eq!(
            pressure_span_summary(&[1008.2, 1012.9, 1010.0]),
            "1008..1013hPa"
        );
    }

    fn dt(hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, 20)
            .expect("valid date")
            .and_hms_opt(hour, 0, 0)
            .expect("valid time")
    }

    fn hour(hour: u32, precip_mm: Option<f32>) -> HourlyForecast {
        HourlyForecast {
            time: dt(hour),
            temperature_2m_c: None,
            weather_code: None,
            is_day: None,
            relative_humidity_2m: None,
            precipitation_probability: None,
            precipitation_mm: precip_mm,
            rain_mm: None,
            snowfall_cm: None,
            wind_speed_10m: None,
            wind_gusts_10m: None,
            pressure_msl_hpa: None,
            visibility_m: None,
            cloud_cover: None,
            cloud_cover_low: None,
            cloud_cover_mid: None,
            cloud_cover_high: None,
        }
    }
}

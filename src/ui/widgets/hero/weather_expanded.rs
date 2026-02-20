#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::must_use_candidate
)]

use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::fmt::Write as _;

use crate::{
    app::state::AppState,
    domain::weather::{
        AirQualityCategory, ForecastBundle, HourlyForecast, convert_temp, round_temp,
        round_wind_speed, weather_code_to_category, weather_label_for_time,
    },
    ui::theme::{Theme, condition_color},
};

mod trends;

use super::weather::{
    HeroScale, cloud_layers_from_hourly, compass, format_cloud_layers, format_visibility,
    pressure_trend_marker,
};
use trends::{build_expanded_trend_lines, collect_trend_series};

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
    fetch_context: Option<String>,
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
    precip_probability: String,
    sunrise: String,
    sunset: String,
    aqi: String,
    aqi_category: AirQualityCategory,
    aqi_available: bool,
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
        updated: last_updated_label(state, weather),
        fetch_context: expanded_fetch_context(state),
    }
}

fn freshness_status(state: &AppState, theme: Theme) -> (&'static str, Color) {
    match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => ("Fresh", theme.success),
        crate::resilience::freshness::FreshnessState::Stale => ("⚠ Stale", theme.warning),
        crate::resilience::freshness::FreshnessState::Offline => ("⚠ Offline", theme.danger),
    }
}

fn last_updated_label(state: &AppState, weather: &ForecastBundle) -> String {
    let timezone = weather.location.timezone.as_deref().unwrap_or("--");
    state
        .refresh_meta
        .last_success
        .map(|ts| {
            let local = ts.with_timezone(&Local);
            let mins = state.refresh_meta.age_minutes().unwrap_or(0);
            format!(
                "Last updated {} ({}m ago) · TZ {}",
                local.format("%H:%M"),
                mins.max(0),
                timezone
            )
        })
        .unwrap_or_else(|| format!("Last updated --:-- · TZ {timezone}"))
}

fn build_expanded_metrics_data(state: &AppState, weather: &ForecastBundle) -> ExpandedMetricsData {
    let (cloud_low, cloud_mid, cloud_high) =
        cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
    let (aqi, aqi_category, aqi_available) = expanded_aqi_summary(weather);
    ExpandedMetricsData {
        feels: round_temp(convert_temp(
            weather.current.apparent_temperature_c,
            state.units,
        )),
        dew: round_temp(convert_temp(weather.current.dew_point_2m_c, state.units)),
        wind_dir: compass(weather.current.wind_direction_10m).to_string(),
        wind: round_wind_speed(weather.current.wind_speed_10m),
        gust: round_wind_speed(weather.current.wind_gusts_10m),
        visibility: format_visibility(weather.current.visibility_m),
        pressure: weather.current.pressure_msl_hpa.round() as i32,
        pressure_trend: pressure_trend_marker(&weather.hourly),
        humidity: weather.current.relative_humidity_2m.round() as i32,
        cloud_total: weather.current.cloud_cover.round() as i32,
        cloud_split: format_cloud_layers(cloud_low, cloud_mid, cloud_high),
        uv_today: expanded_uv_today(weather),
        precip_probability: expanded_precip_probability(&weather.hourly),
        sunrise: expanded_sun_time(weather, |day| day.sunrise),
        sunset: expanded_sun_time(weather, |day| day.sunset),
        aqi,
        aqi_category,
        aqi_available,
    }
}

fn expanded_uv_today(weather: &ForecastBundle) -> String {
    weather
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "--".to_string())
}

fn expanded_sun_time(
    weather: &ForecastBundle,
    projection: impl Fn(&crate::domain::weather::DailyForecast) -> Option<chrono::NaiveDateTime>,
) -> String {
    weather
        .daily
        .first()
        .and_then(projection)
        .map(|value| value.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string())
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
    if let Some(fetch_context) = &data.fetch_context {
        top_lines.push(Line::from(Span::styled(
            fetch_context.clone(),
            Style::default().fg(theme.warning),
        )));
    }
    top_lines
}

fn build_expanded_metric_lines(
    data: &ExpandedMetricsData,
    theme: Theme,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    (
        expanded_left_metric_lines(data, theme),
        expanded_right_metric_lines(data, theme),
    )
}

fn expanded_left_metric_lines(data: &ExpandedMetricsData, theme: Theme) -> Vec<Line<'static>> {
    vec![
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
                format!("{}/{} m/s {}", data.wind, data.gust, data.wind_dir),
                Style::default().fg(theme.success),
            ),
            Span::raw("  "),
            Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
            Span::styled(data.visibility.clone(), Style::default().fg(theme.accent)),
        ]),
    ]
}

fn expanded_right_metric_lines(data: &ExpandedMetricsData, theme: Theme) -> Vec<Line<'static>> {
    vec![
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
            Span::raw("  "),
            Span::styled("P ", Style::default().fg(theme.muted_text)),
            Span::styled(
                data.precip_probability.clone(),
                Style::default().fg(theme.info),
            ),
        ]),
        Line::from(vec![
            Span::styled("Sunrise ", Style::default().fg(theme.muted_text)),
            Span::styled(data.sunrise.clone(), Style::default().fg(theme.warning)),
            Span::raw("  "),
            Span::styled("Sunset ", Style::default().fg(theme.muted_text)),
            Span::styled(data.sunset.clone(), Style::default().fg(theme.warning)),
            Span::raw("  "),
            Span::styled("AQI ", Style::default().fg(theme.muted_text)),
            Span::styled(
                data.aqi.clone(),
                Style::default().fg(expanded_aqi_color(data, theme)),
            ),
        ]),
    ]
}

fn expanded_precip_probability(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(12)
        .find_map(|hour| hour.precipitation_probability)
        .map_or_else(
            || "--".to_string(),
            |value| format!("{}%", value.round() as i32),
        )
}

fn expanded_aqi_summary(weather: &ForecastBundle) -> (String, AirQualityCategory, bool) {
    let Some(reading) = weather.air_quality.as_ref() else {
        return ("N/A".to_string(), AirQualityCategory::Unknown, false);
    };

    (
        format!("{} {}", reading.display_value(), reading.category.label()),
        reading.category,
        true,
    )
}

fn expanded_aqi_color(data: &ExpandedMetricsData, theme: Theme) -> Color {
    if !data.aqi_available {
        return theme.muted_text;
    }

    match data.aqi_category {
        AirQualityCategory::Good => theme.success,
        AirQualityCategory::Moderate => theme.warning,
        AirQualityCategory::UnhealthySensitive
        | AirQualityCategory::Unhealthy
        | AirQualityCategory::VeryUnhealthy
        | AirQualityCategory::Hazardous => theme.danger,
        AirQualityCategory::Unknown => theme.muted_text,
    }
}

fn expanded_fetch_context(state: &AppState) -> Option<String> {
    let error = state.last_error.as_ref()?;
    if matches!(
        state.refresh_meta.state,
        crate::resilience::freshness::FreshnessState::Fresh
    ) {
        return None;
    }
    let mut context = format!("Last fetch failed: {}", expanded_summarize_error(error, 72));
    if let Some(retry_secs) = state.refresh_meta.retry_in_seconds() {
        let _ = write!(context, " · retry in {retry_secs}s");
    }
    Some(context)
}

fn expanded_summarize_error(error: &str, max_len: usize) -> String {
    let first_line = error.lines().next().unwrap_or_default();
    let text = first_line.trim();
    if text.chars().count() <= max_len {
        return text.to_string();
    }

    let mut out = String::new();
    for ch in text.chars().take(max_len.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests;

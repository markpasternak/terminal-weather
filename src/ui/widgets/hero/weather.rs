#![allow(
    clippy::comparison_chain,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::must_use_candidate
)]

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::state::{AppMode, AppState},
    domain::weather::{
        AirQualityCategory, ForecastBundle, HourlyForecast, convert_temp, round_temp,
        round_wind_speed, weather_code_to_category, weather_label_for_time,
    },
    ui::theme::{Theme, condition_color},
};

mod loading;
mod metric_lines;
mod metrics;

use metric_lines::{WeatherMetricsData, push_metric_lines};
pub use metrics::{
    cloud_layers_from_hourly, compass, format_cloud_layers, format_visibility,
    pressure_trend_marker,
};

use super::{shared as hero_shared, weather_expanded::render_weather_info_expanded};
use loading::render_loading_choreography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroScale {
    Compact,
    Standard,
    Deluxe,
}

impl HeroScale {
    pub fn for_area(area: Rect) -> Self {
        if area.width >= 84 && area.height >= 14 {
            Self::Deluxe
        } else if area.width >= 56 && area.height >= 9 {
            Self::Standard
        } else {
            Self::Compact
        }
    }

    pub fn compact_metrics(self) -> bool {
        matches!(self, Self::Compact)
    }

    pub fn metric_gap(self) -> &'static str {
        if matches!(self, Self::Deluxe) {
            "    "
        } else {
            "  "
        }
    }

    pub fn chart_left_padding(self) -> u16 {
        if matches!(self, Self::Deluxe) { 10 } else { 12 }
    }

    pub fn chart_max_width(self) -> u16 {
        if matches!(self, Self::Deluxe) {
            120
        } else {
            72
        }
    }
}

pub fn render_weather_info(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
    code: u8,
) {
    let scale = HeroScale::for_area(area);
    if let Some(weather) = &state.weather {
        if area.height >= 13 && area.width >= 48 {
            render_weather_info_expanded(frame, area, state, theme, weather, code);
            return;
        }
        let lines = build_weather_lines(state, weather, theme, code, scale);
        frame.render_widget(Paragraph::new(lines), area);
        return;
    }

    if state.mode == AppMode::Error {
        frame.render_widget(Paragraph::new(build_error_lines(state, theme)), area);
    } else {
        render_loading_choreography(frame, area, state, theme, scale);
    }
}

fn build_weather_lines(
    state: &AppState,
    weather: &ForecastBundle,
    theme: Theme,
    code: u8,
    scale: HeroScale,
) -> Vec<Line<'static>> {
    let mut lines = build_header_lines(state, weather, theme, code, scale);
    let metrics = collect_weather_metrics(state, weather);
    push_metric_lines(
        &mut lines,
        &metrics,
        theme,
        scale.metric_gap(),
        scale.compact_metrics(),
    );
    if let Some((flag, color)) = freshness_flag(state, theme) {
        lines.push(Line::from(Span::styled(
            flag,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(fetch_context) = fetch_context_line(state) {
        lines.push(Line::from(Span::styled(
            fetch_context,
            Style::default().fg(theme.warning),
        )));
    }
    lines.push(Line::from(Span::styled(
        last_updated_label(state, weather),
        Style::default().fg(theme.muted_text),
    )));
    lines
}

fn build_header_lines(
    state: &AppState,
    weather: &ForecastBundle,
    theme: Theme,
    code: u8,
    scale: HeroScale,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let (temp, unit_symbol) = current_temp_display(state, weather);
    let weather_label = weather_label_for_time(code, weather.current.is_day);
    let weather_color = condition_color(&theme, weather_code_to_category(code));
    append_header_main_line(
        &mut lines,
        scale,
        theme,
        temp,
        unit_symbol,
        weather_label,
        weather_color,
    );
    if let Some((high, low)) = weather.high_low(state.units) {
        lines.push(Line::from(Span::styled(
            format!("H:{high}°  L:{low}°"),
            Style::default().fg(theme.text),
        )));
    }
    lines.push(Line::from(Span::styled(
        weather.location.display_name(),
        Style::default().fg(theme.text),
    )));
    lines
}

fn current_temp_display(state: &AppState, weather: &ForecastBundle) -> (i32, &'static str) {
    (
        weather.current_temp(state.units),
        if state.units == crate::domain::weather::Units::Celsius {
            "C"
        } else {
            "F"
        },
    )
}

fn append_header_main_line(
    lines: &mut Vec<Line<'static>>,
    scale: HeroScale,
    theme: Theme,
    temp: i32,
    unit_symbol: &'static str,
    weather_label: &str,
    weather_color: Color,
) {
    if matches!(scale, HeroScale::Deluxe) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{temp}°{unit_symbol}"),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  "),
            Span::styled(
                weather_label.to_string(),
                Style::default()
                    .fg(weather_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        return;
    }
    lines.push(Line::from(vec![Span::styled(
        format!("{temp}°{unit_symbol}"),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::styled(
        weather_label.to_string(),
        Style::default()
            .fg(weather_color)
            .add_modifier(Modifier::BOLD),
    )));
}

fn collect_weather_metrics(state: &AppState, weather: &ForecastBundle) -> WeatherMetricsData {
    let (cloud_low, cloud_mid, cloud_high) =
        cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
    let (aqi, aqi_category, aqi_available) = aqi_summary(weather);
    WeatherMetricsData {
        feels: round_temp(convert_temp(
            weather.current.apparent_temperature_c,
            state.units,
        )),
        humidity: weather.current.relative_humidity_2m.round() as i32,
        dew: round_temp(convert_temp(weather.current.dew_point_2m_c, state.units)),
        wind_dir: compass(weather.current.wind_direction_10m),
        wind: round_wind_speed(weather.current.wind_speed_10m),
        gust: round_wind_speed(weather.current.wind_gusts_10m),
        visibility: format_visibility(weather.current.visibility_m),
        pressure: weather.current.pressure_msl_hpa.round() as i32,
        pressure_trend: pressure_trend_marker(&weather.hourly),
        uv_today: weather
            .daily
            .first()
            .and_then(|d| d.uv_index_max)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "--".to_string()),
        cloud_total: weather.current.cloud_cover.round() as i32,
        cloud_split: format_cloud_layers(cloud_low, cloud_mid, cloud_high),
        precip_probability: next_precip_probability(&weather.hourly),
        aqi,
        aqi_category,
        aqi_available,
    }
}

fn freshness_flag(state: &AppState, theme: Theme) -> Option<(&'static str, Color)> {
    match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => None,
        crate::resilience::freshness::FreshnessState::Stale => Some(("⚠ stale", theme.warning)),
        crate::resilience::freshness::FreshnessState::Offline => Some(("⚠ offline", theme.danger)),
    }
}

fn last_updated_label(state: &AppState, weather: &ForecastBundle) -> String {
    hero_shared::last_updated_label(state, weather, true)
}

fn build_error_lines(state: &AppState, theme: Theme) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        "Unable to load weather",
        Style::default().fg(theme.text),
    ))];
    if let Some(err) = &state.last_error {
        lines.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(theme.muted_text),
        )));
    }
    lines
}

fn fetch_context_line(state: &AppState) -> Option<String> {
    hero_shared::fetch_context_line(state, 68)
}

fn next_precip_probability(hourly: &[HourlyForecast]) -> String {
    hero_shared::next_precip_probability(hourly)
}

fn aqi_summary(weather: &ForecastBundle) -> (String, AirQualityCategory, bool) {
    hero_shared::aqi_summary(weather)
}

#[cfg(test)]
mod tests;

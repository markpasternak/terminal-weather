#![allow(
    clippy::comparison_chain,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::must_use_candidate
)]

use chrono::Local;
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
        ForecastBundle, convert_temp, round_temp, weather_code_to_category, weather_label_for_time,
    },
    ui::theme::{Theme, condition_color},
};

use super::weather_expanded::render_weather_info_expanded;

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

#[derive(Debug)]
struct WeatherMetricsData {
    feels: i32,
    humidity: i32,
    dew: i32,
    wind_dir: &'static str,
    wind: i32,
    gust: i32,
    visibility: String,
    pressure: i32,
    pressure_trend: &'static str,
    uv_today: String,
    cloud_total: i32,
    cloud_split: String,
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
    if scale.compact_metrics() {
        push_compact_metric_lines(&mut lines, &metrics, theme, scale.metric_gap());
    } else {
        push_standard_metric_lines(&mut lines, &metrics, theme, scale.metric_gap());
    }
    if let Some((flag, color)) = freshness_flag(state, theme) {
        lines.push(Line::from(Span::styled(
            flag,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(Line::from(Span::styled(
        last_updated_label(state),
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
    let temp = weather.current_temp(state.units);
    let unit_symbol = if state.units == crate::domain::weather::Units::Celsius {
        "C"
    } else {
        "F"
    };
    let weather_label = weather_label_for_time(code, weather.current.is_day);
    let weather_color = condition_color(&theme, weather_code_to_category(code));
    if matches!(scale, HeroScale::Deluxe) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{temp}°{unit_symbol}"),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  "),
            Span::styled(
                weather_label,
                Style::default()
                    .fg(weather_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            format!("{temp}°{unit_symbol}"),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(Span::styled(
            weather_label,
            Style::default()
                .fg(weather_color)
                .add_modifier(Modifier::BOLD),
        )));
    }
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

fn collect_weather_metrics(state: &AppState, weather: &ForecastBundle) -> WeatherMetricsData {
    let (cloud_low, cloud_mid, cloud_high) =
        cloud_layers_from_hourly(&weather.hourly).unwrap_or((None, None, None));
    WeatherMetricsData {
        feels: round_temp(convert_temp(
            weather.current.apparent_temperature_c,
            state.units,
        )),
        humidity: weather.current.relative_humidity_2m.round() as i32,
        dew: round_temp(convert_temp(weather.current.dew_point_2m_c, state.units)),
        wind_dir: compass(weather.current.wind_direction_10m),
        wind: weather.current.wind_speed_10m.round() as i32,
        gust: weather.current.wind_gusts_10m.round() as i32,
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
            format!("{}/{} km/h {}", data.wind, data.gust, data.wind_dir),
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
}

fn push_standard_metric_lines(
    lines: &mut Vec<Line<'static>>,
    data: &WeatherMetricsData,
    theme: Theme,
    metric_gap: &'static str,
) {
    lines.push(Line::from(vec![
        Span::styled("Feels ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("{}°", data.feels), Style::default().fg(theme.text)),
        Span::raw(metric_gap),
        Span::styled("Dew ", Style::default().fg(theme.muted_text)),
        Span::styled(format!("{}°", data.dew), Style::default().fg(theme.info)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Wind ", Style::default().fg(theme.muted_text)),
        Span::styled(
            format!("{}/{} km/h {}", data.wind, data.gust, data.wind_dir),
            Style::default().fg(theme.success),
        ),
        Span::raw(metric_gap),
        Span::styled("Visibility ", Style::default().fg(theme.muted_text)),
        Span::styled(data.visibility.clone(), Style::default().fg(theme.accent)),
    ]));
    lines.push(Line::from(vec![
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
    ]));
    lines.push(Line::from(vec![
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
    ]));
}

fn freshness_flag(state: &AppState, theme: Theme) -> Option<(&'static str, Color)> {
    match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => None,
        crate::resilience::freshness::FreshnessState::Stale => Some(("⚠ stale", theme.warning)),
        crate::resilience::freshness::FreshnessState::Offline => Some(("⚠ offline", theme.danger)),
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
                "Last updated: {} ({}m ago)",
                local.format("%H:%M"),
                mins.max(0)
            )
        })
        .unwrap_or_else(|| "Last updated: --:--".to_string())
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

pub fn compass(deg: f32) -> &'static str {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let mut idx = ((deg.rem_euclid(360.0) / 45.0).round() as usize) % 8;
    if idx >= DIRS.len() {
        idx = 0;
    }
    DIRS[idx]
}

pub fn format_visibility(meters: f32) -> String {
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

pub fn pressure_trend_marker(hourly: &[crate::domain::weather::HourlyForecast]) -> &'static str {
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

pub fn cloud_layers_from_hourly(
    hourly: &[crate::domain::weather::HourlyForecast],
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

pub fn format_cloud_layers(low: Option<f32>, mid: Option<f32>, high: Option<f32>) -> String {
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

fn render_loading_choreography(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: Theme,
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

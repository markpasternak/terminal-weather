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
use std::fmt::Write as _;

use crate::{
    app::state::{AppMode, AppState},
    domain::weather::{
        AirQualityCategory, ForecastBundle, HourlyForecast, convert_temp, round_temp,
        round_wind_speed, weather_code_to_category, weather_label_for_time,
    },
    ui::theme::{Theme, condition_color},
};

mod loading;

use super::weather_expanded::render_weather_info_expanded;
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
    precip_probability: String,
    aqi: String,
    aqi_category: AirQualityCategory,
    aqi_available: bool,
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

fn freshness_flag(state: &AppState, theme: Theme) -> Option<(&'static str, Color)> {
    match state.refresh_meta.state {
        crate::resilience::freshness::FreshnessState::Fresh => None,
        crate::resilience::freshness::FreshnessState::Stale => Some(("⚠ stale", theme.warning)),
        crate::resilience::freshness::FreshnessState::Offline => Some(("⚠ offline", theme.danger)),
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
                "Last updated: {} local ({}m ago) · City TZ {}",
                local.format("%H:%M"),
                mins.max(0),
                timezone
            )
        })
        .unwrap_or_else(|| format!("Last updated: --:-- local · City TZ {timezone}"))
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
    let error = state.last_error.as_ref()?;
    if matches!(
        state.refresh_meta.state,
        crate::resilience::freshness::FreshnessState::Fresh
    ) {
        return None;
    }
    let mut context = format!("Last fetch failed: {}", summarize_error(error, 68));
    if let Some(retry_secs) = state.refresh_meta.retry_in_seconds() {
        let _ = write!(context, " · retry in {retry_secs}s");
    }
    Some(context)
}

fn summarize_error(error: &str, max_len: usize) -> String {
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

fn next_precip_probability(hourly: &[HourlyForecast]) -> String {
    hourly
        .iter()
        .take(12)
        .find_map(|hour| hour.precipitation_probability)
        .map_or_else(
            || "--".to_string(),
            |value| format!("{}%", value.round() as i32),
        )
}

fn aqi_summary(weather: &ForecastBundle) -> (String, AirQualityCategory, bool) {
    let Some(reading) = weather.air_quality.as_ref() else {
        return ("N/A".to_string(), AirQualityCategory::Unknown, false);
    };

    (
        format!("{} {}", reading.display_value(), reading.category.label()),
        reading.category,
        true,
    )
}

fn aqi_color(data: &WeatherMetricsData, theme: Theme) -> Color {
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

#[cfg(test)]
mod tests {
    use super::{
        compass, fetch_context_line, format_visibility, freshness_flag, last_updated_label,
    };
    use crate::{
        app::state::AppState,
        cli::{Cli, ColorArg, HeroVisualArg, ThemeArg, UnitsArg},
        domain::weather::{
            CurrentConditions, DailyForecast, ForecastBundle, HourlyForecast, Location,
            RefreshMetadata,
        },
        resilience::freshness::FreshnessState,
        ui::theme::{ColorCapability, theme_for},
    };
    use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};

    #[test]
    fn freshness_flag_warns_on_stale() {
        let mut state = AppState::new(&test_cli());
        state.refresh_meta.state = FreshnessState::Stale;
        let theme = theme_for(
            crate::domain::weather::WeatherCategory::Unknown,
            false,
            ColorCapability::TrueColor,
            ThemeArg::Auto,
        );

        let flag = freshness_flag(&state, theme).expect("stale flag");
        assert_eq!(flag.0, "⚠ stale");
    }

    #[test]
    fn last_updated_label_includes_timezone() {
        let mut state = AppState::new(&test_cli());
        state.refresh_meta = RefreshMetadata {
            last_success: Some(Utc::now() - Duration::minutes(3)),
            ..RefreshMetadata::default()
        };
        let weather = sample_bundle();
        let label = last_updated_label(&state, &weather);
        assert!(label.contains("TZ Europe/Stockholm"));
    }

    #[test]
    fn compass_rounds_directions() {
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(44.0), "NE");
        assert_eq!(compass(225.0), "SW");
    }

    #[test]
    fn format_visibility_formats_km() {
        assert_eq!(format_visibility(12_345.0), "12.3km");
        assert_eq!(format_visibility(20_100.0), "20km");
        assert_eq!(format_visibility(-1.0), "--");
    }

    #[test]
    fn fetch_context_line_shows_retry_when_available() {
        let mut state = AppState::new(&test_cli());
        state.refresh_meta.state = FreshnessState::Offline;
        state.last_error = Some("network timeout".to_string());
        state.refresh_meta.schedule_retry_in(30);

        let line = fetch_context_line(&state).expect("fetch context line");
        assert!(line.contains("network timeout"));
        assert!(line.contains("retry in"));
    }

    fn test_cli() -> Cli {
        Cli {
            city: Some("Stockholm".to_string()),
            units: UnitsArg::Celsius,
            fps: 30,
            no_animation: true,
            reduced_motion: false,
            no_flash: true,
            ascii_icons: false,
            emoji_icons: false,
            color: ColorArg::Auto,
            no_color: false,
            hourly_view: None,
            theme: ThemeArg::Auto,
            hero_visual: HeroVisualArg::AtmosCanvas,
            country_code: None,
            lat: None,
            lon: None,
            refresh_interval: 600,
            demo: false,
            one_shot: false,
        }
    }

    fn sample_bundle() -> ForecastBundle {
        ForecastBundle {
            location: Location {
                name: "Stockholm".to_string(),
                latitude: 59.3293,
                longitude: 18.0686,
                country: Some("Sweden".to_string()),
                admin1: Some("Stockholm".to_string()),
                timezone: Some("Europe/Stockholm".to_string()),
                population: None,
            },
            current: CurrentConditions {
                temperature_2m_c: 7.0,
                relative_humidity_2m: 72.0,
                apparent_temperature_c: 5.0,
                dew_point_2m_c: 2.0,
                weather_code: 3,
                precipitation_mm: 0.0,
                cloud_cover: 40.0,
                pressure_msl_hpa: 1008.0,
                visibility_m: 10_000.0,
                wind_speed_10m: 10.0,
                wind_gusts_10m: 15.0,
                wind_direction_10m: 180.0,
                is_day: true,
                high_today_c: Some(8.0),
                low_today_c: Some(1.0),
            },
            hourly: vec![HourlyForecast {
                time: NaiveDateTime::parse_from_str("2026-02-12T10:00", "%Y-%m-%dT%H:%M")
                    .expect("valid time"),
                temperature_2m_c: Some(7.0),
                weather_code: Some(3),
                is_day: Some(true),
                relative_humidity_2m: Some(72.0),
                precipitation_probability: Some(35.0),
                precipitation_mm: Some(0.0),
                rain_mm: Some(0.0),
                snowfall_cm: Some(0.0),
                wind_speed_10m: Some(10.0),
                wind_gusts_10m: Some(15.0),
                pressure_msl_hpa: Some(1008.0),
                visibility_m: Some(10_000.0),
                cloud_cover: Some(40.0),
                cloud_cover_low: Some(20.0),
                cloud_cover_mid: Some(30.0),
                cloud_cover_high: Some(35.0),
            }],
            daily: vec![DailyForecast {
                date: NaiveDate::from_ymd_opt(2026, 2, 12).expect("valid date"),
                weather_code: Some(3),
                temperature_max_c: Some(8.0),
                temperature_min_c: Some(1.0),
                sunrise: None,
                sunset: None,
                uv_index_max: Some(2.0),
                precipitation_probability_max: Some(35.0),
                precipitation_sum_mm: Some(0.0),
                rain_sum_mm: Some(0.0),
                snowfall_sum_cm: Some(0.0),
                precipitation_hours: Some(0.0),
                wind_gusts_10m_max: Some(15.0),
                daylight_duration_s: Some(32_000.0),
                sunshine_duration_s: Some(18_000.0),
            }],
            air_quality: None,
            fetched_at: Utc::now(),
        }
    }
}

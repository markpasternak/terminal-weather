#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::domain::weather::{ForecastBundle, WeatherCategory, weather_code_to_category};
use crate::ui::widgets::landmark::shared::{
    compass_arrow, compass_short, fit_lines, fit_lines_centered,
};
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

#[derive(Debug)]
struct GaugeData {
    temp_c: i32,
    humidity: f32,
    pressure: f32,
    wind: f32,
    gust: f32,
    wind_direction_10m: f32,
    uv: f32,
    vis_km: f32,
    precip_now: f32,
    cloud: f32,
    meter_w: usize,
    left_col_width: usize,
    right_trend_width: usize,
    sunrise: String,
    sunset: String,
    temp_track: Vec<f32>,
    precip_track: Vec<f32>,
    gust_track: Vec<f32>,
}

#[must_use]
pub fn scene_for_gauge_cluster(bundle: &ForecastBundle, width: u16, height: u16) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);
    let data = collect_gauge_data(bundle, w);
    let left_lines = build_left_lines(&data);
    let mut lines = if w >= 74 && h >= 9 {
        let right_lines = build_right_lines(&data, category, bundle.current.is_day);
        merge_columns(&left_lines, &right_lines, data.left_col_width)
    } else {
        left_lines
    };

    if h >= 12 && w < 74 {
        append_wind_direction_block(&mut lines, data.wind_direction_10m);
    }

    LandmarkScene {
        label: "Gauge Cluster · Live Instruments".to_string(),
        lines: if h >= 12 {
            fit_lines_centered(lines, w, h)
        } else {
            fit_lines(lines, w, h)
        },
        tint: tint_for_category(category),
        context_line: Some(gauge_context_line(&data)),
    }
}

fn gauge_context_line(data: &GaugeData) -> String {
    // Pick the most notable insight
    if data.uv > 7.0 {
        format!("⚠ UV {:.1} very high — limit sun exposure", data.uv)
    } else if data.gust > 50.0 {
        format!(
            "⚠ Gusts {} m/s — secure loose objects",
            crate::domain::weather::round_wind_speed(data.gust)
        )
    } else if data.uv > 5.0 {
        format!("UV {:.1} high · sunscreen advised", data.uv)
    } else if data.gust > 30.0 {
        format!(
            "Gusty winds {} m/s · dress for wind",
            crate::domain::weather::round_wind_speed(data.gust)
        )
    } else if data.vis_km < 1.0 {
        format!("Visibility {:.1}km · drive carefully", data.vis_km)
    } else if data.precip_now > 0.5 {
        format!("Active precipitation {:.1}mm/h", data.precip_now)
    } else if data.humidity > 85.0 {
        format!("Humidity {:.0}% · feels muggy", data.humidity)
    } else {
        format!("Pressure {:.0} hPa · conditions stable", data.pressure)
    }
}

fn collect_gauge_data(bundle: &ForecastBundle, width: usize) -> GaugeData {
    let current = &bundle.current;
    let left_col_width = left_column_width(width);
    let trend_width = width.saturating_sub(left_col_width + 12).clamp(8, 28);
    let (sunrise, sunset) = gauge_sun_times(bundle);
    let (temp_track, precip_track, gust_track) = gauge_tracks(bundle);

    GaugeData {
        temp_c: current.temperature_2m_c.round() as i32,
        humidity: current.relative_humidity_2m.clamp(0.0, 100.0),
        pressure: current.pressure_msl_hpa,
        wind: current.wind_speed_10m.max(0.0),
        gust: current.wind_gusts_10m.max(0.0),
        wind_direction_10m: current.wind_direction_10m,
        uv: bundle
            .daily
            .first()
            .and_then(|day| day.uv_index_max)
            .unwrap_or(0.0),
        vis_km: (current.visibility_m / 1000.0).max(0.0),
        precip_now: current.precipitation_mm.max(0.0),
        cloud: current.cloud_cover.clamp(0.0, 100.0),
        meter_w: width.saturating_sub(26).clamp(10, 56),
        left_col_width,
        right_trend_width: trend_width.saturating_sub(6),
        sunrise,
        sunset,
        temp_track,
        precip_track,
        gust_track,
    }
}

fn gauge_sun_times(bundle: &ForecastBundle) -> (String, String) {
    let sunrise = bundle
        .daily
        .first()
        .and_then(|day| day.sunrise)
        .map_or_else(
            || "--:--".to_string(),
            |value| value.format("%H:%M").to_string(),
        );
    let sunset = bundle.daily.first().and_then(|day| day.sunset).map_or_else(
        || "--:--".to_string(),
        |value| value.format("%H:%M").to_string(),
    );
    (sunrise, sunset)
}

fn gauge_tracks(bundle: &ForecastBundle) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let temp_track = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|hour| hour.temperature_2m_c)
        .collect::<Vec<_>>();
    let precip_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|hour| hour.precipitation_mm.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let gust_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|hour| hour.wind_gusts_10m.unwrap_or(0.0))
        .collect::<Vec<_>>();
    (temp_track, precip_track, gust_track)
}

fn left_column_width(width: usize) -> usize {
    if width >= 86 {
        width.saturating_mul(58) / 100
    } else if width >= 74 {
        width.saturating_mul(62) / 100
    } else {
        width
    }
}

fn build_left_lines(data: &GaugeData) -> Vec<String> {
    let temp_norm = ((data.temp_c as f32 + 20.0) / 60.0).clamp(0.0, 1.0);
    let pressure_norm = ((data.pressure - 970.0) / 70.0).clamp(0.0, 1.0);
    let uv_norm = (data.uv / 12.0).clamp(0.0, 1.0);
    let vis_norm = (data.vis_km / 12.0).clamp(0.0, 1.0);
    // Temperature comfort zone threshold at ~20°C = norm 0.67
    let uv_warn = if data.uv > 5.0 { " ⚠" } else { "" };
    vec![
        "Current conditions".to_string(),
        format!(
            "Temp   {} {:>4}°C",
            meter_with_threshold(temp_norm, data.meter_w, Some(0.67)),
            data.temp_c
        ),
        format!(
            "Humid  {} {:>4.0}%",
            meter_with_threshold(data.humidity / 100.0, data.meter_w, None),
            data.humidity
        ),
        format!(
            "Press  {} {:>4.0}hPa",
            meter_with_threshold(pressure_norm, data.meter_w, None),
            data.pressure
        ),
        format!(
            "UV Idx {} {:>4.1}{uv_warn}",
            meter_with_threshold(uv_norm, data.meter_w, Some(0.42)),
            data.uv
        ),
        format!(
            "Visib  {} {:>4.1}km",
            meter_with_threshold(vis_norm, data.meter_w, None),
            data.vis_km
        ),
        format!(
            "Wind   {:>2} {:>4} m/s  gust {:>3}",
            compass_arrow(data.wind_direction_10m),
            crate::domain::weather::round_wind_speed(data.wind),
            crate::domain::weather::round_wind_speed(data.gust)
        ),
    ]
}

fn build_right_lines(data: &GaugeData, category: WeatherCategory, is_day: bool) -> Vec<String> {
    vec![
        "24-Hour Overview".to_string(),
        format!("Condition {}", scene_name(category, is_day)),
        format!(
            "Cloud {:>3.0}%   Rain now {:>3.1}mm",
            data.cloud, data.precip_now
        ),
        format!("Sun arc {} -> {}", data.sunrise, data.sunset),
        format!(
            "24h Temp  {} {}",
            sparkline_annotated(&data.temp_track, data.right_trend_width, "°"),
            temp_range_label(&data.temp_track)
        ),
        format!(
            "24h Rain  {} {}",
            sparkline_annotated(&data.precip_track, data.right_trend_width, ""),
            precip_range_label(&data.precip_track)
        ),
        format!(
            "24h Gust  {} {}",
            sparkline_annotated(&data.gust_track, data.right_trend_width, ""),
            gust_range_label(&data.gust_track)
        ),
        format!("Visibility {:>4.1}km", data.vis_km),
        wind_compass_box(data.wind_direction_10m),
    ]
}

fn merge_columns(left: &[String], right: &[String], left_col_width: usize) -> Vec<String> {
    let mut merged = Vec::with_capacity(left.len().max(right.len()));
    for idx in 0..left.len().max(right.len()) {
        let left_line = left.get(idx).map_or("", String::as_str);
        let right_line = right.get(idx).map_or("", String::as_str);
        merged.push(format!("{left_line:<left_col_width$}  {right_line}"));
    }
    merged
}

fn append_wind_direction_block(lines: &mut Vec<String>, wind_direction_10m: f32) {
    lines.push(String::new());
    lines.push(wind_compass_box(wind_direction_10m));
}

fn wind_compass_box(wind_direction_10m: f32) -> String {
    format!(
        "Wind ┌───┐ {} {}",
        compass_arrow(wind_direction_10m),
        compass_short(wind_direction_10m)
    )
}

fn meter_with_threshold(norm: f32, width: usize, threshold: Option<f32>) -> String {
    let width = width.max(4);
    let fill = (norm.clamp(0.0, 1.0) * width as f32).round() as usize;
    let thresh_pos = threshold.map(|t| (t.clamp(0.0, 1.0) * width as f32).round() as usize);
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for idx in 0..width {
        let ch = if thresh_pos == Some(idx) {
            '|'
        } else if idx < fill {
            '█'
        } else if idx == fill {
            '▓'
        } else if idx == fill.saturating_add(1) {
            '▒'
        } else {
            '·'
        };
        bar.push(ch);
    }
    bar.push(']');
    bar
}

fn sparkline_annotated(values: &[f32], width: usize, _suffix: &str) -> String {
    sparkline_blocks(values, width)
}

fn temp_range_label(values: &[f32]) -> String {
    range_label(values, "°")
}

fn precip_range_label(values: &[f32]) -> String {
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    if max > 0.0 {
        format!("{:.0}mm", max)
    } else {
        String::new()
    }
}

fn gust_range_label(values: &[f32]) -> String {
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    if max > 0.0 {
        format!("{}m/s", crate::domain::weather::round_wind_speed(max))
    } else {
        String::new()
    }
}

fn range_label(values: &[f32], suffix: &str) -> String {
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("{:.0}{suffix}–{:.0}{suffix}", min, max)
}

fn sparkline_blocks(values: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (max - min).max(0.001);
    (0..width)
        .map(|idx| {
            let src = (idx * values.len() / width).min(values.len().saturating_sub(1));
            let norm = ((values[src] - min) / span).clamp(0.0, 1.0);
            BARS[(norm * (BARS.len() - 1) as f32).round() as usize]
        })
        .collect()
}

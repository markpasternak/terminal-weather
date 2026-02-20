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
    }
}

fn collect_gauge_data(bundle: &ForecastBundle, width: usize) -> GaugeData {
    let current = &bundle.current;
    let left_col_width = left_column_width(width);
    let trend_width = width.saturating_sub(left_col_width + 12).clamp(8, 28);

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
        sunrise: bundle
            .daily
            .first()
            .and_then(|d| d.sunrise)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string()),
        sunset: bundle
            .daily
            .first()
            .and_then(|d| d.sunset)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string()),
        temp_track: bundle
            .hourly
            .iter()
            .take(24)
            .filter_map(|h| h.temperature_2m_c)
            .collect::<Vec<_>>(),
        precip_track: bundle
            .hourly
            .iter()
            .take(24)
            .map(|h| h.precipitation_mm.unwrap_or(0.0))
            .collect::<Vec<_>>(),
        gust_track: bundle
            .hourly
            .iter()
            .take(24)
            .map(|h| h.wind_gusts_10m.unwrap_or(0.0))
            .collect::<Vec<_>>(),
    }
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
    vec![
        "Current conditions".to_string(),
        format!(
            "Temp   {} {:>4}C",
            meter(temp_norm, data.meter_w),
            data.temp_c
        ),
        format!(
            "Humid  {} {:>4.0}%",
            meter(data.humidity / 100.0, data.meter_w),
            data.humidity
        ),
        format!(
            "Press  {} {:>4.0}hPa",
            meter(pressure_norm, data.meter_w),
            data.pressure
        ),
        format!("UV Idx {} {:>4.1}", meter(uv_norm, data.meter_w), data.uv),
        format!(
            "Visib  {} {:>4.1}km",
            meter(vis_norm, data.meter_w),
            data.vis_km
        ),
        format!(
            "Wind   {:>2} {:>4.0} km/h  gust {:>3.0}",
            compass_arrow(data.wind_direction_10m),
            data.wind,
            data.gust
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
            "24h Temp  {}",
            sparkline_blocks(&data.temp_track, data.right_trend_width)
        ),
        format!(
            "24h Rain  {}",
            sparkline_blocks(&data.precip_track, data.right_trend_width)
        ),
        format!(
            "24h Gust  {}",
            sparkline_blocks(&data.gust_track, data.right_trend_width)
        ),
        format!("Visibility {:>4.1}km", data.vis_km),
        "Wind direction".to_string(),
        "    N".to_string(),
        wind_direction_row(data.wind_direction_10m),
        "    S".to_string(),
    ]
}

fn merge_columns(left: &[String], right: &[String], left_col_width: usize) -> Vec<String> {
    let mut merged = Vec::with_capacity(left.len().max(right.len()));
    for idx in 0..left.len().max(right.len()) {
        let left_line = left.get(idx).map(String::as_str).unwrap_or("");
        let right_line = right.get(idx).map(String::as_str).unwrap_or("");
        merged.push(format!(
            "{left_line:<left_col_width$}  {right_line}",
            left_col_width = left_col_width
        ));
    }
    merged
}

fn append_wind_direction_block(lines: &mut Vec<String>, wind_direction_10m: f32) {
    lines.push("".to_string());
    lines.push("Wind direction".to_string());
    lines.push("    N".to_string());
    lines.push(wind_direction_row(wind_direction_10m));
    lines.push("    S".to_string());
}

fn wind_direction_row(wind_direction_10m: f32) -> String {
    format!(
        "  W {} E   dir {}",
        if compass_arrow(wind_direction_10m) == '←' {
            '◉'
        } else {
            '+'
        },
        compass_short(wind_direction_10m)
    )
}

fn meter(norm: f32, width: usize) -> String {
    let width = width.max(4);
    let fill = (norm.clamp(0.0, 1.0) * width as f32).round() as usize;
    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for idx in 0..width {
        let ch = if idx < fill {
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

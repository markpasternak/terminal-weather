use crate::domain::weather::{ForecastBundle, weather_code_to_category};
use crate::ui::widgets::landmark::shared::{
    compass_arrow, compass_short, fit_lines, fit_lines_centered,
};
use crate::ui::widgets::landmark::{LandmarkScene, scene_name, tint_for_category};

pub fn scene_for_gauge_cluster(bundle: &ForecastBundle, width: u16, height: u16) -> LandmarkScene {
    let w = width as usize;
    let h = height as usize;
    let category = weather_code_to_category(bundle.current.weather_code);

    let current = &bundle.current;
    let temp_c = current.temperature_2m_c.round() as i32;
    let humidity = current.relative_humidity_2m.clamp(0.0, 100.0);
    let pressure = current.pressure_msl_hpa;
    let wind = current.wind_speed_10m.max(0.0);
    let gust = current.wind_gusts_10m.max(0.0);
    let uv = bundle
        .daily
        .first()
        .and_then(|day| day.uv_index_max)
        .unwrap_or(0.0);
    let vis_km = (current.visibility_m / 1000.0).max(0.0);

    let meter_w = w.saturating_sub(26).clamp(10, 56);
    let pressure_norm = ((pressure - 970.0) / 70.0).clamp(0.0, 1.0);
    let uv_norm = (uv / 12.0).clamp(0.0, 1.0);
    let temp_norm = ((temp_c as f32 + 20.0) / 60.0).clamp(0.0, 1.0);
    let vis_norm = (vis_km / 12.0).clamp(0.0, 1.0);
    let precip_now = current.precipitation_mm.max(0.0);
    let cloud = current.cloud_cover.clamp(0.0, 100.0);
    let left_col_width = if w >= 86 {
        w.saturating_mul(58) / 100
    } else if w >= 74 {
        w.saturating_mul(62) / 100
    } else {
        w
    };

    let sunrise = bundle
        .daily
        .first()
        .and_then(|d| d.sunrise)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());
    let sunset = bundle
        .daily
        .first()
        .and_then(|d| d.sunset)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string());

    let temp_track = bundle
        .hourly
        .iter()
        .take(24)
        .filter_map(|h| h.temperature_2m_c)
        .collect::<Vec<_>>();
    let precip_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|h| h.precipitation_mm.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let gust_track = bundle
        .hourly
        .iter()
        .take(24)
        .map(|h| h.wind_gusts_10m.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let trend_width = w.saturating_sub(left_col_width + 12).clamp(8, 28);
    let right_trend_width = trend_width.saturating_sub(6);

    let left_lines = [
        "Current conditions".to_string(),
        format!("Temp   {} {:>4}C", meter(temp_norm, meter_w), temp_c),
        format!(
            "Humid  {} {:>4.0}%",
            meter(humidity / 100.0, meter_w),
            humidity
        ),
        format!(
            "Press  {} {:>4.0}hPa",
            meter(pressure_norm, meter_w),
            pressure
        ),
        format!("UV Idx {} {:>4.1}", meter(uv_norm, meter_w), uv),
        format!("Visib  {} {:>4.1}km", meter(vis_norm, meter_w), vis_km),
        format!(
            "Wind   {:>2} {:>4.0} km/h  gust {:>3.0}",
            compass_arrow(current.wind_direction_10m),
            wind,
            gust
        ),
    ];
    let mut lines = if w >= 74 && h >= 9 {
        let right_lines = [
            "24-Hour Overview".to_string(),
            format!("Condition {}", scene_name(category, bundle.current.is_day)),
            format!("Cloud {:>3.0}%   Rain now {:>3.1}mm", cloud, precip_now),
            format!("Sun arc {sunrise} -> {sunset}"),
            format!(
                "24h Temp  {}",
                sparkline_blocks(&temp_track, right_trend_width)
            ),
            format!(
                "24h Rain  {}",
                sparkline_blocks(&precip_track, right_trend_width)
            ),
            format!(
                "24h Gust  {}",
                sparkline_blocks(&gust_track, right_trend_width)
            ),
            format!("Visibility {:>4.1}km", vis_km),
            "Wind direction".to_string(),
            "    N".to_string(),
            format!(
                "  W {} E   dir {}",
                if compass_arrow(current.wind_direction_10m) == '←' {
                    '◉'
                } else {
                    '+'
                },
                compass_short(current.wind_direction_10m)
            ),
            "    S".to_string(),
        ];

        let mut merged = Vec::with_capacity(left_lines.len().max(right_lines.len()));
        for idx in 0..left_lines.len().max(right_lines.len()) {
            let left = left_lines.get(idx).map(String::as_str).unwrap_or("");
            let right = right_lines.get(idx).map(String::as_str).unwrap_or("");
            merged.push(format!(
                "{left:<left_col_width$}  {right}",
                left_col_width = left_col_width
            ));
        }
        merged
    } else {
        left_lines.to_vec()
    };

    if h >= 12 && w < 74 {
        lines.push("".to_string());
        lines.push("Wind direction".to_string());
        lines.push("    N".to_string());
        lines.push(format!(
            "  W {} E   dir {}",
            if compass_arrow(current.wind_direction_10m) == '←' {
                '◉'
            } else {
                '+'
            },
            compass_short(current.wind_direction_10m)
        ));
        lines.push("    S".to_string());
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

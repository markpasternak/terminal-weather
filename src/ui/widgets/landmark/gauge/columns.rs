use crate::domain::weather::WeatherCategory;
use crate::ui::widgets::landmark::scene_name;
use crate::ui::widgets::landmark::shared::{compass_arrow, compass_short};

use super::data::GaugeData;
use super::meters::{
    gust_range_label, meter_with_threshold, precip_range_label, sparkline_annotated,
    temp_range_label,
};

pub(super) fn build_left_lines(data: &GaugeData) -> Vec<String> {
    let temp_norm = ((data.temp_c as f32 + 20.0) / 60.0).clamp(0.0, 1.0);
    let pressure_norm = ((data.pressure - 970.0) / 70.0).clamp(0.0, 1.0);
    let uv_norm = (data.uv / 12.0).clamp(0.0, 1.0);
    let vis_norm = (data.vis_km / 12.0).clamp(0.0, 1.0);
    let uv_warn = if data.uv > 5.0 { " ⚠" } else { "" };
    vec![
        "Current conditions".to_string(),
        format!(
            "Temp   {} {:>4}°{}",
            meter_with_threshold(temp_norm, data.meter_w, Some(0.67)),
            data.temp_display,
            data.temp_unit
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

pub(super) fn build_right_lines(
    data: &GaugeData,
    category: WeatherCategory,
    is_day: bool,
) -> Vec<String> {
    vec![
        "24-Hour Overview".to_string(),
        format!("Condition {}", scene_name(category, is_day)),
        format!(
            "Cloud {:>3.0}%   Precip now {:>3.1}mm",
            data.cloud, data.precip_now
        ),
        format!("Sun arc {} -> {}", data.sunrise, data.sunset),
        format!(
            "24h Temp  {} {}",
            sparkline_annotated(&data.temp_track_display, data.right_trend_width, "°"),
            temp_range_label(&data.temp_track_display)
        ),
        format!(
            "24h Precip {} {}",
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

pub(super) fn merge_columns(
    left: &[String],
    right: &[String],
    left_col_width: usize,
) -> Vec<String> {
    let mut merged = Vec::with_capacity(left.len().max(right.len()));
    for idx in 0..left.len().max(right.len()) {
        let left_line = left.get(idx).map_or("", String::as_str);
        let right_line = right.get(idx).map_or("", String::as_str);
        merged.push(format!("{left_line:<left_col_width$}  {right_line}"));
    }
    merged
}

pub(super) fn append_wind_direction_block(lines: &mut Vec<String>, wind_direction_10m: f32) {
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

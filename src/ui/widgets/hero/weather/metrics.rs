use crate::domain::weather::HourlyForecast;

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

pub fn pressure_trend_marker(hourly: &[HourlyForecast]) -> &'static str {
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
    hourly: &[HourlyForecast],
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

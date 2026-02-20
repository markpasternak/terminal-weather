pub(super) fn format_duration_hm(seconds: f32) -> String {
    let total_minutes = (seconds.max(0.0) / 60.0).round() as i64;
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    format!("{h:02}:{m:02}")
}

pub(super) fn profile_bar(values: &[f32], width: usize) -> String {
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

pub(super) fn precipitation_cue(day: &crate::domain::weather::DailyForecast) -> String {
    let precip = day.precipitation_sum_mm.unwrap_or(0.0);
    let snow = day.snowfall_sum_cm.unwrap_or(0.0);
    if snow >= 1.0 {
        return format!("snow {snow:.1}cm");
    }
    if precip >= 6.0 {
        return format!("wet {precip:.1}mm");
    }
    if precip >= 1.0 {
        return format!("light rain {precip:.1}mm");
    }
    "mostly dry".to_string()
}

pub(super) fn gust_cue(gust: f32) -> Option<String> {
    if gust >= 45.0 {
        return Some(format!("gusty {}km/h", gust.round() as i32));
    }
    if gust >= 30.0 {
        return Some(format!("breezy {}km/h", gust.round() as i32));
    }
    None
}

pub(super) fn sunlight_cue(day: &crate::domain::weather::DailyForecast) -> Option<&'static str> {
    let ratio = match (day.sunshine_duration_s, day.daylight_duration_s) {
        (Some(sun), Some(daylight)) if daylight > 0.0 => Some((sun / daylight).clamp(0.0, 1.0)),
        _ => None,
    }?;

    if ratio >= 0.65 {
        Some("bright")
    } else if ratio <= 0.25 {
        Some("dim")
    } else {
        None
    }
}
